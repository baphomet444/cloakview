//! All `#[tauri::command]` wrappers — the thin IPC surface. Pure logic
//! lives in `db::*`, `secrets`, `settings`, `listen`, `answers`; this file
//! only adapts those to Tauri's command/state/event plumbing.

use crate::db::{conversations, notes, Db};
use crate::error::{AppError, IpcError};
use crate::events::{ANSWER_DELTA_EVENT, ANSWER_DONE_EVENT, ERROR_EVENT};
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
    let mut s = state.settings.write().expect("settings lock");
    settings::apply_patch(&mut s, patch);
    settings::save_settings(&app, &s).map_err(IpcError::from)?;
    Ok(s.clone())
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
    let model = state.settings.read().expect("settings lock").realtime_model.clone();
    let mut guard = state.listen_session.lock().await;
    if guard.is_some() {
        return Ok(()); // already listening; idempotent.
    }
    let session = listen::start(model, app).map_err(IpcError::from)?;
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
    let api_key = secrets::get_api_key().map_err(IpcError::from)?;
    let model = state.settings.read().expect("settings lock").answers_model.clone();
    let context: Vec<crate::transcript::Turn> = state.transcript.lock().await.clone();

    let mut answer_guard = state.answer_task.lock().await;
    if let Some(prev) = answer_guard.take() {
        prev.abort(); // a new question supersedes whatever was still streaming.
    }

    let (mut rx, handle) = crate::answers::client::stream_answer(api_key, model, context, question);
    *answer_guard = Some(handle);
    drop(answer_guard);

    // Spec event names, not the enum's own `kind` tag (see `listen.rs`'s
    // doc for the same rule on the realtime side): one Tauri event per
    // `AnswersEvent` variant, bare payload, no discrimination needed on
    // the frontend. `AnswersEvent::Error` shares `"error"` with the
    // realtime side — the spec lists one generic error event, not a
    // separate per-source one.
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                crate::answers::AnswersEvent::Delta { text } => {
                    let _ = app.emit(ANSWER_DELTA_EVENT, text);
                }
                crate::answers::AnswersEvent::Done => {
                    let _ = app.emit(ANSWER_DONE_EVENT, ());
                }
                crate::answers::AnswersEvent::Error { message } => {
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

/// Opens an `http`/`https` URL in the default browser. Any other scheme (or a
/// malformed URL) is rejected as `InvalidUrl` before anything is launched.
#[tauri::command]
pub async fn open_external(url: String) -> Result<(), IpcError> {
    platform::open_external(&url).await.map_err(IpcError::from)
}

/// Turns the window `label` into an overlay panel like the hub (excluded from capture,
/// over full-screen Spaces, non-activating) and shows it. The hub calls this for
/// Ask/Notes/Settings, which it creates hidden. Safe to call again on the same window.
///
/// AppKit must be touched on the main thread. The command is `async` (so it runs on a
/// worker, never on the main thread) and hops there with `run_on_main_thread`: a sync
/// command blocking on the main thread for work queued to the main thread would
/// deadlock.
#[tauri::command]
pub async fn promote_overlay(app: AppHandle, label: String) -> Result<(), IpcError> {
    let window = app
        .get_webview_window(&label)
        .ok_or_else(|| AppError::Audio(format!("no window labelled {label:?}")))?;
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.run_on_main_thread(move || {
        let _ = tx.send(crate::windows_ext::make_overlay(&window));
    })
    .map_err(|e| AppError::Audio(e.to_string()))?;
    rx.await
        .map_err(|_| AppError::Audio("main thread dropped the overlay request".into()))?
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
