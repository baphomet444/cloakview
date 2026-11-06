use crate::error::AppResult;
use crate::state::Settings;
use std::path::PathBuf;

pub fn settings_path(app_data_dir: &std::path::Path) -> PathBuf {
    app_data_dir.join("settings.json")
}

pub fn load_from(path: &std::path::Path) -> Settings {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

pub fn save_to(path: &std::path::Path, s: &Settings) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| crate::error::AppError::Serialize(e.to_string()))?;
    }
    let raw = serde_json::to_string_pretty(s)?;
    std::fs::write(path, raw).map_err(|e| crate::error::AppError::Serialize(e.to_string()))?;
    Ok(())
}

use tauri::Manager;

pub fn load_settings(app: &tauri::AppHandle) -> Settings {
    let dir = app.path().app_data_dir().unwrap_or_default();
    load_from(&settings_path(&dir))
}

pub fn save_settings(app: &tauri::AppHandle, s: &Settings) -> AppResult<()> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| crate::error::AppError::Serialize(e.to_string()))?;
    save_to(&settings_path(&dir), s)
}

#[derive(Default, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SettingsPatch {
    pub language: Option<String>,
    pub theme: Option<String>,
    pub opacity: Option<f64>,
    pub answers_model: Option<String>,
    pub realtime_model: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_corrupt_fallback() {
        let dir = std::env::temp_dir().join("cloakview-settings-test");
        let path = settings_path(&dir);
        let _ = std::fs::remove_file(&path);

        // missing file -> defaults
        let s = load_from(&path);
        assert_eq!(s.language, "en");
        assert_eq!(s.answers_model, "gpt-4o-mini");

        // save + load roundtrip
        let mut s2 = s.clone();
        s2.opacity = 0.5;
        save_to(&path, &s2).unwrap();
        let loaded = load_from(&path);
        assert!((loaded.opacity - 0.5).abs() < f64::EPSILON);

        // corrupt file -> defaults, no panic
        std::fs::write(&path, "{not json").unwrap();
        let s3 = load_from(&path);
        assert_eq!(s3.language, "en");
    }
}
