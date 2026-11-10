pub mod answers;
pub mod audio;
mod commands;
mod db;
pub mod detect;
mod error;
mod events;
mod hotkeys;
mod listen;
mod platform;
pub mod realtime;
mod secrets;
mod settings;
mod state;
pub mod transcript;
mod windows_ext;

use tauri::Manager;

pub use error::{AppError, AppResult, IpcError};
pub use state::{AppState, Settings};

/// The whole IPC surface, declared in exactly one place.
///
/// The one token list below feeds both `tauri::generate_handler!` (what the
/// frontend can actually invoke) and `REGISTERED_COMMANDS` (what the
/// `ipc_surface_tests` compare against design.md §5), so the two cannot
/// drift: a rename or an omission here either fails to compile (the fn is
/// gone) or fails that test (the name no longer matches the spec).
macro_rules! ipc_commands {
    ($($cmd:ident),+ $(,)?) => {
        #[cfg(test)]
        const REGISTERED_COMMANDS: &[&str] = &[$(stringify!($cmd)),+];

        fn invoke_handler() -> impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static {
            tauri::generate_handler![$(commands::$cmd),+]
        }
    };
}

ipc_commands![
    save_api_key,
    get_api_key_status,
    clear_api_key,
    get_settings,
    save_settings,
    create_note,
    list_notes,
    update_note,
    delete_note,
    create_conversation,
    add_message,
    list_messages,
    delete_conversation,
    start_listen,
    stop_listen,
    request_answer,
    stop_answer,
    get_mic_status,
    get_app_info,
    open_external,
    promote_overlay,
];

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_nspanel::init())
        .invoke_handler(invoke_handler())
        .setup(|app| {
            let s = settings::load_settings(app.handle());
            let shortcuts = s.shortcuts.clone();
            let data_dir = app.path().app_data_dir()?;
            let db = db::Db::open(&data_dir.join("cloakview.db")).map_err(|e| e.to_string())?;
            app.manage(AppState {
                settings: std::sync::RwLock::new(s),
                db: std::sync::Mutex::new(Some(db)),
                transcript: tokio::sync::Mutex::new(Vec::new()),
                listen_session: tokio::sync::Mutex::new(None),
                answer_task: tokio::sync::Mutex::new(None),
            });
            let hub = app.get_webview_window("hub").ok_or("hub window missing")?;
            windows_ext::make_overlay(&hub, false).map_err(|e| e.to_string())?;
            // After the state is managed (the handlers read it). A shortcut that cannot
            // be registered (taken by another app, say) is logged and skipped; it never
            // stops the app from starting.
            hotkeys::register_all(app.handle(), &shortcuts);
            #[cfg(all(debug_assertions, target_os = "macos"))]
            windows_ext::spawn_overlay_probe(hub.clone());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building cloakview")
        .run(|app, event| {
            // Clicking the Dock icon after `Close` hid the whole overlay: bring the hub
            // back. `make_overlay` is idempotent and ends in `order_front_regardless`,
            // never making the hub key, and `Reopen` is delivered on the main thread,
            // which it requires.
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { has_visible_windows, .. } = event {
                if !has_visible_windows {
                    if let Some(hub) = app.get_webview_window("hub") {
                        let _ = crate::windows_ext::make_overlay(&hub, false);
                    }
                }
            }
            #[cfg(not(target_os = "macos"))]
            let _ = (app, event);
        });
}

#[cfg(test)]
mod ipc_surface_tests {
    use super::REGISTERED_COMMANDS;
    use std::collections::BTreeSet;

    /// The commands design.md §5 ("Comandos IPC") names, transcribed by hand.
    ///
    /// - §5 only says "CRUD conversaciones" without names; the four
    ///   conversation commands below are the ones agreed for it.
    /// - §5 also mentions notes "folders/tags" without naming commands, and
    ///   none exist yet (there is a `folders` table, no IPC). When they are
    ///   added, this constant must grow in the same change.
    ///
    /// Being a manual transcription, it will not notice the spec itself
    /// changing — only the registered set drifting away from *this* list.
    const SPEC_COMMANDS: &[&str] = &[
        // secrets
        "get_api_key_status",
        "save_api_key",
        "clear_api_key",
        // listen / answer
        "start_listen",
        "stop_listen",
        "get_mic_status",
        "request_answer",
        "stop_answer",
        // notes
        "create_note",
        "list_notes",
        "update_note",
        "delete_note",
        // conversations
        "create_conversation",
        "add_message",
        "list_messages",
        "delete_conversation",
        // settings / app
        "get_settings",
        "save_settings",
        "get_app_info",
        "open_external",
        // windows (not in design.md §5: added for the overlay panels)
        "promote_overlay",
    ];

    #[test]
    fn registered_commands_are_exactly_the_spec_commands() {
        let registered: BTreeSet<&str> = REGISTERED_COMMANDS.iter().copied().collect();
        let spec: BTreeSet<&str> = SPEC_COMMANDS.iter().copied().collect();

        assert_eq!(
            registered.len(),
            REGISTERED_COMMANDS.len(),
            "a command is listed twice in ipc_commands!"
        );
        assert_eq!(spec.len(), SPEC_COMMANDS.len(), "SPEC_COMMANDS lists a name twice");

        let missing: Vec<_> = spec.difference(&registered).collect();
        let unexpected: Vec<_> = registered.difference(&spec).collect();
        assert!(
            missing.is_empty() && unexpected.is_empty(),
            "IPC surface differs from design.md §5.\n  in the spec but not registered: {missing:?}\n  registered but not in the spec: {unexpected:?}"
        );
    }
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
