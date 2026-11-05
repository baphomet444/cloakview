mod error;
mod state;

pub use error::{AppError, AppResult, IpcError};
pub use state::{AppState, Settings};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(AppState::default())
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running cloakview");
}
