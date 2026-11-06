pub mod answers;
pub mod audio;
mod db;
pub mod detect;
mod error;
pub mod realtime;
mod secrets;
mod settings;
mod state;
pub mod transcript;

use tauri::Manager;

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

#[tauri::command]
fn get_settings(state: tauri::State<AppState>) -> Settings {
    state.settings.read().expect("settings lock").clone()
}

#[tauri::command]
fn save_settings_cmd(
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
    patch: settings::SettingsPatch,
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

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            save_api_key,
            has_api_key,
            clear_api_key,
            get_settings,
            save_settings_cmd
        ])
        .setup(|app| {
            let s = settings::load_settings(app.handle());
            let data_dir = app.path().app_data_dir()?;
            let db = db::Db::open(&data_dir.join("cloakview.db")).map_err(|e| e.to_string())?;
            app.manage(AppState {
                settings: std::sync::RwLock::new(s),
                db: std::sync::Mutex::new(Some(db)),
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running cloakview");
}
