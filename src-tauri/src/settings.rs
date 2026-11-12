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
    pub user_keywords: Option<Vec<String>>,
    /// `None` = leave unchanged. A blank string clears the name (a plain
    /// `Option<String>` can't tell "absent" from "set to null" once
    /// deserialized, so blank is the explicit "clear" signal).
    pub user_name: Option<String>,
    /// Replaces all three accelerators at once (a missing key inside falls back to its
    /// default, via `Shortcuts`' `serde(default)`).
    pub shortcuts: Option<crate::hotkeys::Shortcuts>,
    pub include_mic: Option<bool>,
    /// Replaces the whole notes block; blank clears it.
    pub personal_context: Option<String>,
    /// One of `balanced`, `interview`, `sales`, `technical`, `short`. Free-form
    /// in Rust; anything unknown behaves like `balanced`.
    pub answer_style: Option<String>,
    /// "" (off), "en" or "es". Free-form in Rust; anything unknown behaves like off.
    pub translate_to: Option<String>,
}

/// Applies the fields present in `patch` to `s`: `opacity` is clamped to
/// `0.1..=1.0`; `user_name` is trimmed, and a blank value clears it.
pub fn apply_patch(s: &mut Settings, patch: SettingsPatch) {
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
    if let Some(v) = patch.user_keywords {
        s.user_keywords = v;
    }
    if let Some(v) = patch.user_name {
        let v = v.trim();
        s.user_name = (!v.is_empty()).then(|| v.to_string());
    }
    if let Some(v) = patch.shortcuts {
        s.shortcuts = v;
    }
    if let Some(v) = patch.include_mic {
        s.include_mic = v;
    }
    if let Some(v) = patch.personal_context {
        s.personal_context = v.trim().to_string();
    }
    if let Some(v) = patch.answer_style {
        s.answer_style = v;
    }
    if let Some(v) = patch.translate_to {
        s.translate_to = v;
    }
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

    #[test]
    fn patch_sets_trims_and_clears_user_name() {
        let mut s = Settings::default();

        apply_patch(&mut s, SettingsPatch { user_name: Some("  Ana ".into()), ..Default::default() });
        assert_eq!(s.user_name.as_deref(), Some("Ana"));

        // absent from the patch -> untouched
        apply_patch(&mut s, SettingsPatch::default());
        assert_eq!(s.user_name.as_deref(), Some("Ana"));

        // blank -> cleared
        apply_patch(&mut s, SettingsPatch { user_name: Some("   ".into()), ..Default::default() });
        assert_eq!(s.user_name, None);
    }

    #[test]
    fn patch_replaces_personal_context_trimmed() {
        let mut s = Settings::default();
        apply_patch(&mut s, SettingsPatch { personal_context: Some("  Rust engineer.  ".into()), ..Default::default() });
        assert_eq!(s.personal_context, "Rust engineer.");
        apply_patch(&mut s, SettingsPatch { personal_context: Some("  ".into()), ..Default::default() });
        assert_eq!(s.personal_context, "");
    }

    #[test]
    fn patch_replaces_answer_style_verbatim() {
        let mut s = Settings::default();
        assert_eq!(s.answer_style, "balanced");
        apply_patch(&mut s, SettingsPatch { answer_style: Some("interview".into()), ..Default::default() });
        assert_eq!(s.answer_style, "interview");
    }
}
