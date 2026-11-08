pub mod answers;
pub mod audio;
mod commands;
mod db;
pub mod detect;
mod error;
mod listen;
pub mod realtime;
mod secrets;
mod settings;
mod state;
pub mod transcript;

use tauri::Manager;

pub use error::{AppError, AppResult, IpcError};
pub use state::{AppState, Settings};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::save_api_key,
            commands::has_api_key,
            commands::clear_api_key,
            commands::get_settings,
            commands::save_settings_cmd,
            commands::create_note_cmd,
            commands::list_notes_cmd,
            commands::update_note_cmd,
            commands::delete_note_cmd,
            commands::create_conversation_cmd,
            commands::add_message_cmd,
            commands::list_messages_cmd,
            commands::delete_conversation_cmd,
            commands::start_listen,
            commands::stop_listen,
            commands::request_answer,
            commands::stop_answer,
        ])
        .setup(|app| {
            let s = settings::load_settings(app.handle());
            let data_dir = app.path().app_data_dir()?;
            let db = db::Db::open(&data_dir.join("cloakview.db")).map_err(|e| e.to_string())?;
            app.manage(AppState {
                settings: std::sync::RwLock::new(s),
                db: std::sync::Mutex::new(Some(db)),
                transcript: tokio::sync::Mutex::new(Vec::new()),
                listen_session: tokio::sync::Mutex::new(None),
                answer_task: tokio::sync::Mutex::new(None),
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running cloakview");
}

#[cfg(test)]
mod contract_tests {
    //! These types cross the IPC boundary to the TypeScript front (Task 12).
    //! Only the JSON matters. A `matches!` or a field-name grep cannot catch
    //! a missing `rename_all`; comparing real JSON can.

    use crate::transcript::Turn;

    #[test]
    fn turn_uses_camel_case() {
        let t = Turn {
            speaker: "remote".into(),
            text: "Hola".into(),
            is_user_turn: false,
            ts: 1_700_000_000,
        };
        assert_eq!(
            serde_json::to_value(&t).unwrap(),
            serde_json::json!({
                "speaker": "remote",
                "text": "Hola",
                "isUserTurn": false,
                "ts": 1700000000
            })
        );
    }

    #[test]
    fn note_uses_camel_case() {
        let n = crate::db::notes::Note {
            id: "n1".into(),
            title: "T".into(),
            content: "C".into(),
            tags: vec![],
            favorite: false,
            archived: false,
            folder_id: Some("f1".into()),
            created_at: 1,
            updated_at: 2,
        };
        let j = serde_json::to_value(&n).unwrap();
        assert!(j.get("folderId").is_some(), "folderId missing: {j}");
        assert!(j.get("createdAt").is_some(), "createdAt missing: {j}");
        assert!(j.get("folder_id").is_none(), "snake_case leaked: {j}");
    }
}
