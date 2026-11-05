mod error;
mod secrets;
mod state;

pub use error::{AppError, AppResult, IpcError};
pub use state::{AppState, Settings};

#[tauri::command]
fn save_api_key(key: String) -> Result<(), IpcError> {
    secrets::save_api_key(key).map_err(IpcError::from)
}

#[tauri::command]
fn has_api_key() -> Result<bool, IpcError> {
    secrets::has_api_key().map_err(IpcError::from)
}

#[tauri::command]
fn clear_api_key() -> Result<(), IpcError> {
    secrets::clear_api_key().map_err(IpcError::from)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            save_api_key,
            has_api_key,
            clear_api_key
        ])
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running cloakview");
}
