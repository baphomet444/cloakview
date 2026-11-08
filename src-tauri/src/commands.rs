//! All `#[tauri::command]` wrappers — the thin IPC surface. Pure logic
//! lives in `db::*`, `secrets`, `settings`, `listen`, `answers`; this file
//! only adapts those to Tauri's command/state/event plumbing.

use crate::db::{conversations, notes, Db};
use crate::error::{AppError, IpcError};
use crate::listen;
use crate::secrets;
use crate::settings::{self, SettingsPatch};
use crate::state::{AppState, Settings};
use tauri::{AppHandle, Emitter, State};

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
pub fn has_api_key() -> Result<bool, IpcError> {
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
pub fn save_settings_cmd(
    app: AppHandle,
    state: State<AppState>,
    patch: SettingsPatch,
) -> Result<Settings, IpcError> {
    let mut s = state.settings.write().expect("settings lock");
    if let Some(v) = patch.language {
        s.language = v;
    }
    if let Some(v) = patch.theme {
        s.theme = v;
    }
    if let Some(v) = patch.opacity {
        s.opacity = v.clamp(0.1, 1.0);
    }
    if let Some(v) = patch.answers_model {
        s.answers_model = v;
    }
    if let Some(v) = patch.realtime_model {
        s.realtime_model = v;
    }
    settings::save_settings(&app, &s).map_err(IpcError::from)?;
    Ok(s.clone())
}

// --- notes (Task 5) ---

#[tauri::command]
pub fn create_note_cmd(state: State<AppState>, input: notes::NewNote) -> Result<notes::Note, IpcError> {
    with_db(&state, |db| notes::create(db, input))
}

#[tauri::command]
pub fn list_notes_cmd(
    state: State<AppState>,
    filter: Option<notes::NoteFilter>,
) -> Result<Vec<notes::Note>, IpcError> {
    with_db(&state, |db| notes::list(db, filter.unwrap_or_default()))
}

#[tauri::command]
pub fn update_note_cmd(state: State<AppState>, note: notes::Note) -> Result<notes::Note, IpcError> {
    with_db(&state, |db| notes::update(db, &note))
}

#[tauri::command]
pub fn delete_note_cmd(state: State<AppState>, id: String) -> Result<(), IpcError> {
    with_db(&state, |db| notes::delete(db, &id))
}

// --- conversations (Task 6) ---

#[tauri::command]
pub fn create_conversation_cmd(
    state: State<AppState>,
    title: String,
) -> Result<conversations::Conversation, IpcError> {
    with_db(&state, |db| conversations::create_conversation(db, title))
}

#[tauri::command]
pub fn add_message_cmd(
    state: State<AppState>,
    input: conversations::NewMessage,
) -> Result<conversations::ConversationMessage, IpcError> {
    with_db(&state, |db| conversations::add_message(db, input))
}

#[tauri::command]
pub fn list_messages_cmd(
    state: State<AppState>,
    conversation_id: String,
) -> Result<Vec<conversations::ConversationMessage>, IpcError> {
    with_db(&state, |db| conversations::list_messages(db, &conversation_id))
}

#[tauri::command]
pub fn delete_conversation_cmd(state: State<AppState>, id: String) -> Result<(), IpcError> {
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
                    let _ = app.emit("answer-delta", text);
                }
                crate::answers::AnswersEvent::Done => {
                    let _ = app.emit("answer-done", ());
                }
                crate::answers::AnswersEvent::Error { message } => {
                    let _ = app.emit("error", message);
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
