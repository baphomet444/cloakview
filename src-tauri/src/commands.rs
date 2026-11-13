//! All `#[tauri::command]` wrappers — the thin IPC surface. Pure logic
//! lives in `db::*`, `secrets`, `settings`, `listen`, `answers`; this file
//! only adapts those to Tauri's command/state/event plumbing.

use crate::db::{conversations, notes, sessions, Db};
use crate::error::{AppError, IpcError};
use crate::events::{
    ANSWER_DELTA_EVENT, ANSWER_DONE_EVENT, ANSWER_STARTED_EVENT, ERROR_EVENT,
    SETTINGS_CHANGED_EVENT,
};
use crate::listen;
use crate::platform;
use crate::secrets;
use crate::settings::{self, SettingsPatch};
use crate::state::{AppState, Settings};
use tauri::{AppHandle, Emitter, Manager, State};

/// Runs `f` against the open database, mapping "not initialized yet" (it's
/// set up once in `lib.rs`'s `.setup()`, before any command can run) and
/// any `AppError` it raises to the typed `IpcError` the frontend expects.
fn with_db<T>(
    state: &State<'_, AppState>,
    f: impl FnOnce(&Db) -> crate::error::AppResult<T>,
) -> Result<T, IpcError> {
    let guard = state.db.lock().expect("db lock");
    let db = guard
        .as_ref()
        .ok_or_else(|| AppError::Db("database not initialized".into()))?;
    f(db).map_err(IpcError::from)
}

// --- secrets (Task 3) ---

#[tauri::command]
pub fn save_api_key(key: String) -> Result<(), IpcError> {
    secrets::save_api_key(key).map_err(IpcError::from)
}

#[tauri::command]
pub fn get_api_key_status() -> Result<bool, IpcError> {
    secrets::has_api_key().map_err(IpcError::from)
}

#[tauri::command]
pub fn clear_api_key() -> Result<(), IpcError> {
    secrets::clear_api_key().map_err(IpcError::from)
}

// --- settings (Task 4) ---

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Settings {
    state.settings.read().expect("settings lock").clone()
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: State<AppState>,
    patch: SettingsPatch,
) -> Result<Settings, IpcError> {
    let saved = {
        let mut s = state.settings.write().expect("settings lock");
        let before = s.shortcuts.clone();
        settings::apply_patch(&mut s, patch);
        settings::save_settings(&app, &s).map_err(IpcError::from)?;
        (s.clone(), before)
    };
    let (s, before) = saved;
    // Outside the settings lock. A binding that fails is skipped and shown in the hub's
    // error line, never an error here: the settings are already saved.
    if s.shortcuts != before {
        crate::hotkeys::apply(&app, &s.shortcuts);
    }
    // Every window the hub has ever opened stays alive (shown/hidden, never destroyed),
    // so each needs telling: without this, a window already open when the theme or
    // opacity changes elsewhere would stay stale until the app restarts.
    let _ = app.emit(SETTINGS_CHANGED_EVENT, s.clone());
    Ok(s)
}

// --- notes (Task 5) ---

#[tauri::command]
pub fn create_note(state: State<AppState>, input: notes::NewNote) -> Result<notes::Note, IpcError> {
    with_db(&state, |db| notes::create(db, input))
}

#[tauri::command]
pub fn list_notes(
    state: State<AppState>,
    filter: Option<notes::NoteFilter>,
) -> Result<Vec<notes::Note>, IpcError> {
    with_db(&state, |db| notes::list(db, filter.unwrap_or_default()))
}

#[tauri::command]
pub fn update_note(state: State<AppState>, note: notes::Note) -> Result<notes::Note, IpcError> {
    with_db(&state, |db| notes::update(db, &note))
}

#[tauri::command]
pub fn delete_note(state: State<AppState>, id: String) -> Result<(), IpcError> {
    with_db(&state, |db| notes::delete(db, &id))
}

// --- conversations (Task 6) ---

#[tauri::command]
pub fn create_conversation(
    state: State<AppState>,
    title: String,
) -> Result<conversations::Conversation, IpcError> {
    with_db(&state, |db| conversations::create_conversation(db, title))
}

#[tauri::command]
pub fn add_message(
    state: State<AppState>,
    input: conversations::NewMessage,
) -> Result<conversations::ConversationMessage, IpcError> {
    with_db(&state, |db| conversations::add_message(db, input))
}

#[tauri::command]
pub fn list_messages(
    state: State<AppState>,
    conversation_id: String,
) -> Result<Vec<conversations::ConversationMessage>, IpcError> {
    with_db(&state, |db| conversations::list_messages(db, &conversation_id))
}

#[tauri::command]
pub fn delete_conversation(state: State<AppState>, id: String) -> Result<(), IpcError> {
    with_db(&state, |db| conversations::delete_conversation(db, &id))
}

// --- listen (Task 11) ---

#[tauri::command]
pub async fn start_listen(app: AppHandle, state: State<'_, AppState>) -> Result<(), IpcError> {
    let (model, include_mic) = {
        let s = state.settings.read().expect("settings lock");
        (s.realtime_model.clone(), s.include_mic)
    };
    let mut guard = state.listen_session.lock().await;
    if guard.is_some() {
        return Ok(()); // already listening; idempotent.
    }
    if include_mic && !state.is_pro() {
        // Both-sides listening is a CloakView Pro feature; free listens to the call only.
        return Err(IpcError::from(AppError::ProRequired));
    }
    if include_mic && platform::mic_status() != platform::MicStatus::Authorized {
        // Not allowed yet: this shows the system prompt the first time. The user answers it and presses Listen again.
        platform::request_mic_access();
        crate::diag::log("start_listen: microphone not allowed yet");
        return Err(IpcError::from(AppError::MicPermissionDenied));
    }
    let session = listen::start(model, include_mic, app).map_err(|e| {
        crate::diag::log(&format!("start_listen failed: {e}"));
        IpcError::from(e)
    })?;
    crate::diag::log("start_listen ok");
    state.session_log.lock().expect("session log lock").begin(crate::db::now_ms());
    *guard = Some(session);
    Ok(())
}

#[tauri::command]
pub async fn stop_listen(state: State<'_, AppState>) -> Result<(), IpcError> {
    let session = state.listen_session.lock().await.take();
    if let Some(session) = session {
        session.stop().await;
    }
    Ok(())
}

// --- answers (Task 10 wiring) ---

#[tauri::command]
pub async fn request_answer(
    app: AppHandle,
    state: State<'_, AppState>,
    question: Option<String>,
) -> Result<(), IpcError> {
    start_answer(app, &state, question, None).await
}

/// Reads the screen: takes one screenshot of the main display and streams the answer
/// to the question or task on it, over the same events as `request_answer`. Emits
/// `answer-started` once the screenshot is taken, so Ask shows it is working.
#[tauri::command]
pub async fn read_screen(app: AppHandle, state: State<'_, AppState>) -> Result<(), IpcError> {
    // Pro gate first: fail before taking a screenshot that could not be sent anyway.
    state.license.read().expect("license lock").require_pro().map_err(IpcError::from)?;
    secrets::get_api_key().map_err(IpcError::from)?;
    let image = tokio::task::spawn_blocking(crate::screen::capture)
        .await
        .map_err(|e| IpcError::from(AppError::Audio(e.to_string())))?
        .map_err(|e| {
            crate::diag::log(&format!("read_screen failed: {e}"));
            IpcError::from(e)
        })?;
    crate::diag::log(&format!("screen captured ({} KB)", image.len() * 3 / 4 / 1024));
    let _ = app.emit(ANSWER_STARTED_EVENT, "screen");
    start_answer(app, &state, None, Some(image)).await
}

/// The shared body of `request_answer` and `read_screen`.
async fn start_answer(
    app: AppHandle,
    state: &State<'_, AppState>,
    question: Option<String>,
    image: Option<String>,
) -> Result<(), IpcError> {
    let api_key = secrets::get_api_key().map_err(IpcError::from)?;
    let (model, personal_context, answer_style) = {
        let s = state.settings.read().expect("settings lock");
        (s.answers_model.clone(), s.personal_context.clone(), s.answer_style.clone())
    };
    // Personal context is CloakView Pro: free users' notes stay on this Mac.
    let about_user = state.is_pro().then_some(personal_context).filter(|c| !c.trim().is_empty());
    let context: Vec<crate::transcript::Turn> = state.transcript.lock().await.clone();

    let mut answer_guard = state.answer_task.lock().await;
    if let Some(prev) = answer_guard.take() {
        prev.abort(); // a new question supersedes whatever was still streaming.
    }

    // What this suggestion answers: the typed question, or the last thing heard on the call.
    let prompt = if image.is_some() {
        "(read from the screen)".to_string()
    } else {
        question
            .clone()
            .or_else(|| state.session_log.lock().expect("session log lock").last_heard().map(str::to_string))
            .unwrap_or_default()
    };
    let (mut rx, handle) = crate::answers::client::stream_answer(
        api_key,
        model,
        crate::answers::client::AnswerRequest {
            context,
            question,
            image,
            pro: state.is_pro(),
            about_user,
            style: answer_style,
        },
    );
    *answer_guard = Some(handle);
    drop(answer_guard);

    // Spec event names, not the enum's own `kind` tag (see `listen.rs`'s
    // doc for the same rule on the realtime side): one Tauri event per
    // `AnswersEvent` variant, bare payload, no discrimination needed on
    // the frontend. `AnswersEvent::Error` shares `"error"` with the
    // realtime side — the spec lists one generic error event, not a
    // separate per-source one.
    crate::diag::log("answer requested");
    tokio::spawn(async move {
        // For the diagnostics log: how much was answered, never what.
        let mut chars = 0usize;
        let mut answer = String::new();
        while let Some(event) = rx.recv().await {
            match event {
                crate::answers::AnswersEvent::Delta { text } => {
                    chars += text.chars().count();
                    answer.push_str(&text);
                    let _ = app.emit(ANSWER_DELTA_EVENT, text);
                }
                crate::answers::AnswersEvent::Done => {
                    crate::diag::log(&format!("answer done ({chars} chars)"));
                    app.state::<AppState>().session_log.lock().expect("session log lock").suggestion(
                        crate::db::now_ms(),
                        &prompt,
                        &answer,
                    );
                    let _ = app.emit(ANSWER_DONE_EVENT, ());
                }
                crate::answers::AnswersEvent::Error { message } => {
                    crate::diag::log(&format!("answer error: {message}"));
                    let _ = app.emit(ERROR_EVENT, message);
                }
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_answer(state: State<'_, AppState>) -> Result<(), IpcError> {
    if let Some(handle) = state.answer_task.lock().await.take() {
        handle.abort();
    }
    Ok(())
}

// --- interview record ---

/// What `finish_session` reports, so the hub can offer to save the interview.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub heard: usize,
    pub suggestions: usize,
    /// Whether the suggested answers were saved as a note.
    pub note_saved: bool,
}

/// Closes the interview record once listening has stopped: the suggested answers
/// become a note, and the whole interview is kept for `save_session_txt`. Calling it
/// again, or with nothing recorded, reports zeros. CloakView Pro also gets the
/// after-meeting summary: generated in the background and saved as a note when
/// it is ready (`summary-saved`).
#[tauri::command]
pub fn finish_session(app: AppHandle, state: State<AppState>) -> Result<SessionSummary, IpcError> {
    let finished = state.session_log.lock().expect("session log lock").finish(crate::db::now_ms());
    let Some(mut session) = finished else {
        return Ok(SessionSummary { heard: 0, suggestions: 0, note_saved: false });
    };
    // Saving the suggested answers as a note is CloakView Pro (meeting notes
    // collection); free users keep their own manual notes only.
    if session.suggestion_count() > 0 && state.is_pro() {
        let note = with_db(&state, |db| {
            notes::create(
                db,
                notes::NewNote {
                    title: crate::session::title(session.started_ms),
                    content: crate::session::note_body(&session),
                    tags: vec!["interview".into()],
                    folder_id: None,
                },
            )
        })?;
        session.auto_note_id = Some(note.id);
    }
    let summary = SessionSummary {
        heard: session.heard_count(),
        suggestions: session.suggestion_count(),
        note_saved: session.auto_note_id.is_some(),
    };
    // CloakView Pro keeps every interview in the history window. Free users get
    // the in-memory record and the save offer, but nothing is persisted.
    if state.is_pro() {
        match with_db(&state, |db| sessions::save(db, &session)) {
            Ok(saved) => crate::diag::log(&format!("interview kept in history ({})", saved.id)),
        // `IpcError` is only `Serialize`; its JSON is the log line (same as hotkeys).
        Err(e) => crate::diag::log(&format!(
            "keeping the interview in history failed: {}",
            serde_json::to_string(&e).unwrap_or_default()
        )),
        }
        // The after-meeting summary, generated in the background: it takes a few
        // seconds and nothing may wait for it.
        if session.heard_count() > 0 {
            spawn_summary(app, &state, &session);
        }
    }
    crate::diag::log(&format!(
        "interview finished: {} heard, {} suggestions, note saved: {}",
        summary.heard, summary.suggestions, summary.note_saved
    ));
    *state.last_session.lock().expect("last session lock") = Some(session);
    Ok(summary)
}

/// Kicks off the after-meeting summary (CloakView Pro): one non-streaming
/// request over the interview record, saved as a note; the hub is told via
/// `summary-saved`. Every failure is a log line — the interview itself is
/// already saved by now, so the summary is best-effort.
fn spawn_summary(app: AppHandle, state: &State<'_, AppState>, session: &crate::session::FinishedSession) {
    let Ok(api_key) = secrets::get_api_key() else {
        crate::diag::log("after-meeting summary skipped: no api key");
        return;
    };
    let model = state.settings.read().expect("settings lock").answers_model.clone();
    let record = crate::session::summary_input(session);
    let title = format!("{} — summary", crate::session::title(session.started_ms));
    let started_ms = session.started_ms;
    // `tauri::async_runtime::spawn`, not `tokio::spawn`: this runs on a sync
    // command thread, which has no tokio context of its own.
    tauri::async_runtime::spawn(async move {
        match crate::answers::client::summarize(&api_key, &model, &record).await {
            Ok(text) => {
                let note = {
                    let state = app.state::<AppState>();
                    let guard = state.db.lock().expect("db lock");
                    guard.as_ref().and_then(|db| {
                        notes::create(
                            db,
                            notes::NewNote {
                                title,
                                content: text,
                                tags: vec!["summary".into(), "interview".into()],
                                folder_id: None,
                            },
                        )
                        .ok()
                    })
                };
                if note.is_some() {
                    crate::diag::log(&format!("after-meeting summary saved (interview {started_ms})"));
                    let _ = app.emit(crate::events::SUMMARY_SAVED_EVENT, ());
                }
            }
            Err(e) => crate::diag::log(&format!("after-meeting summary failed: {e}")),
        }
    });
}

// --- session history (CloakView Pro) ---

/// Every past interview, newest first. `query` filters by title and content;
/// `None` (or blank) lists all.
#[tauri::command]
pub fn list_sessions(state: State<AppState>, query: Option<String>) -> Result<Vec<sessions::SessionSummary>, IpcError> {
    state.license.read().expect("license lock").require_pro().map_err(IpcError::from)?;
    with_db(&state, |db| sessions::list(db, query.as_deref()))
}

/// One interview, whole: the conversation and the suggested answers in order.
#[tauri::command]
pub fn get_session(state: State<AppState>, id: String) -> Result<sessions::SessionDetail, IpcError> {
    state.license.read().expect("license lock").require_pro().map_err(IpcError::from)?;
    with_db(&state, |db| {
        sessions::get(db, &id)?.ok_or_else(|| AppError::Db(format!("no session {id}")))
    })
}

#[tauri::command]
pub fn delete_session(state: State<AppState>, id: String) -> Result<(), IpcError> {
    state.license.read().expect("license lock").require_pro().map_err(IpcError::from)?;
    with_db(&state, |db| sessions::delete(db, &id))
}

/// Asks where to save the last interview and writes it there as a text file: the
/// conversation, then the notes written while it ran. Resolves to the path, or `None`
/// if the user closed the dialog.
#[tauri::command]
pub async fn save_session_txt(app: AppHandle, state: State<'_, AppState>) -> Result<Option<String>, IpcError> {
    use tauri_plugin_dialog::DialogExt;

    // Saving the interview to a file is CloakView Pro.
    state.license.read().expect("license lock").require_pro().map_err(IpcError::from)?;
    let session = state.last_session.lock().expect("last session lock").clone();
    let Some(session) = session else { return Ok(None) };
    let lines: Vec<crate::session::NoteLine> = with_db(&state, |db| {
        Ok(notes::list(db, notes::NoteFilter::default())?
            .into_iter()
            .filter(|n| n.updated_at >= session.started_ms && session.auto_note_id.as_deref() != Some(n.id.as_str()))
            .map(|n| crate::session::NoteLine { title: n.title, content: n.content })
            .collect())
    })?;
    let text = crate::session::render_txt(&session, &lines);
    let name = crate::session::file_name(session.started_ms);

    let picked = tokio::task::spawn_blocking(move || {
        app.dialog().file().set_file_name(name).add_filter("Text", &["txt"]).blocking_save_file()
    })
    .await
    .map_err(|e| IpcError::from(AppError::Audio(e.to_string())))?;
    let Some(picked) = picked else { return Ok(None) };
    let path = picked.into_path().map_err(|e| IpcError::from(AppError::OpenFailed(e.to_string())))?;
    std::fs::write(&path, text).map_err(|e| IpcError::from(AppError::from(e)))?;
    crate::diag::log("interview saved to a file");
    Ok(Some(path.display().to_string()))
}

// --- CloakView Pro license ---

/// The stored license, for the Settings window. `Inactive` on a clean install.
#[tauri::command]
pub fn get_license_status(state: State<AppState>) -> crate::license::License {
    state.license.read().expect("license lock").clone()
}

/// Activates the key for this Mac with Lemon Squeezy and unlocks Pro. Rejects with
/// `License` (message: a `license::Rejection` code) when Lemon Squeezy refuses the
/// key, or the key is for another product, or Lemon Squeezy cannot be reached.
#[tauri::command]
pub async fn activate_license(
    app: AppHandle,
    state: State<'_, AppState>,
    key: String,
) -> Result<crate::license::License, IpcError> {
    let license = crate::license::activate(&key, &crate::license::instance_name())
        .await
        .map_err(|r| IpcError::from(AppError::from(r)))?;
    store_license(&app, &state, &license)?;
    crate::diag::log("license activated");
    Ok(license)
}

/// Releases this Mac's activation on Lemon Squeezy (so the key can move to another
/// Mac), then locks Pro and deletes the stored key. When Lemon Squeezy cannot be
/// reached the license is kept, so the slot is never silently lost.
#[tauri::command]
pub async fn deactivate_license(app: AppHandle, state: State<'_, AppState>) -> Result<crate::license::License, IpcError> {
    let current = state.license.read().expect("license lock").clone();
    if let crate::license::License::Active { key, instance_id } = &current {
        crate::license::deactivate(key, instance_id)
            .await
            .map_err(|r| IpcError::from(AppError::from(r)))?;
    }
    let inactive = crate::license::License::Inactive;
    store_license(&app, &state, &inactive)?;
    crate::diag::log("license deactivated");
    Ok(inactive)
}

/// Puts `license` in the app state and in `license.json`.
pub(crate) fn store_license(app: &AppHandle, state: &AppState, license: &crate::license::License) -> Result<(), IpcError> {
    *state.license.write().expect("license lock") = license.clone();
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Serialize(e.to_string()))?;
    crate::license::save_to(&crate::license::license_path(&dir), license).map_err(IpcError::from)
}

// --- app / platform (design.md §5) ---

/// What `get_app_info` reports. `version` is the Tauri package version —
/// `tauri.conf.json`'s `version` (which falls back to the crate's).
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
}

fn app_info(info: &tauri::PackageInfo) -> AppInfo {
    AppInfo { name: info.name.clone(), version: info.version.to_string() }
}

/// Microphone permission, read from the OS without opening a stream (and
/// without prompting).
#[tauri::command]
pub fn get_mic_status() -> platform::MicStatus {
    platform::mic_status()
}

#[tauri::command]
pub fn get_app_info(app: AppHandle) -> AppInfo {
    app_info(app.package_info())
}

/// Opens an `http`/`https` URL in the default browser, or a `mailto:` link in
/// Mail. Any other scheme (or a malformed URL) is rejected as `InvalidUrl`
/// before anything is launched.
#[tauri::command]
pub async fn open_external(url: String) -> Result<(), IpcError> {
    platform::open_external(&url).await.map_err(IpcError::from)
}

/// The global shortcuts the last registration could not take (another app holds the
/// combo, or it is invalid). The hub calls it once it loads: failures at startup happen
/// before its page listens for `shortcut-failed`, so the event alone would lose them.
#[tauri::command]
pub fn shortcut_failures(state: State<AppState>) -> Vec<String> {
    state.shortcut_failures.lock().expect("shortcut failures lock").clone()
}

/// Turns the window `label` into an overlay panel like the hub (excluded from capture,
/// over full-screen Spaces, non-activating) and shows it. The hub calls this for every
/// feature window (Ask, Listen, Notes, History, Settings), which it creates hidden.
/// Safe to call again on the same window.
/// Unlike the hub, these windows stay resizable so their content fits.
///
/// AppKit must be touched on the main thread. The command is `async`, so it runs on a
/// worker, and hops to the main thread with `windows_ext::on_main_thread`.
#[tauri::command]
pub async fn promote_overlay(app: AppHandle, label: String) -> Result<(), IpcError> {
    let window = app
        .get_webview_window(&label)
        .ok_or_else(|| AppError::Audio(format!("no window labelled {label:?}")))?;
    crate::windows_ext::on_main_thread(&app, move |_| {
        crate::windows_ext::make_overlay(&window, true)
    })
    .await
        .map_err(IpcError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `get_app_info` reports what `AppHandle::package_info()` holds. Where
    /// Tauri gets that from (`tauri.conf.json`'s `version`, else the crate's)
    /// is Tauri's own logic; what is ours — and tested — is the mapping.
    /// (`generate_context!()` can't be called a second time to fetch the real
    /// one: it defines a global `_EMBED_INFO_PLIST` symbol and `run()` already
    /// owns it.)
    #[test]
    fn app_info_maps_name_and_version_from_package_info() {
        let package = tauri::PackageInfo {
            name: "cloakview".into(),
            version: "1.2.3-beta.4".parse().unwrap(),
            authors: "",
            description: "",
            crate_name: "cloakview",
        };
        assert_eq!(
            app_info(&package),
            AppInfo { name: "cloakview".into(), version: "1.2.3-beta.4".into() }
        );
    }

    #[test]
    fn app_info_serializes_as_name_and_version() {
        let info = AppInfo { name: "cloakview".into(), version: "0.1.0".into() };
        assert_eq!(
            serde_json::to_value(info).unwrap(),
            serde_json::json!({"name": "cloakview", "version": "0.1.0"})
        );
    }
}
