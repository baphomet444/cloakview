use crate::error::AppResult;
use crate::state::Settings;
use std::path::PathBuf;

pub fn settings_path(app_data_dir: &std::path::Path) -> PathBuf {
    app_data_dir.join("settings.json")
}

/// Missing or unreadable file → defaults. Shortcuts saved with the old ⌘⇧H/⌘⇧L
/// defaults move to the new ones (`Shortcuts::migrate_old_defaults`); the file itself
/// is rewritten the next time settings are saved.
pub fn load_from(path: &std::path::Path) -> Settings {
    let mut s: Settings = std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default();
    s.shortcuts.migrate_old_defaults();
    s
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
    pub realtime_model: Option<String>,
    /// How long to wait after the interviewer pauses before firing
    /// `"turn-detected"`. Clamped to 0..=5000 ms on the Rust side.
    pub reply_delay_ms: Option<u64>,
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
    /// "" (detect), "en" or "es". Free-form in Rust; anything unknown behaves like detect.
    pub call_language: Option<String>,
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
    if let Some(v) = patch.realtime_model {
        s.realtime_model = v;
    }
    if let Some(v) = patch.reply_delay_ms {
        s.reply_delay_ms = v.clamp(0, 5000);
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
    if let Some(v) = patch.call_language {
        s.call_language = v;
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
        assert_eq!(s.active_provider, "openai");

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

    /// A file saved under an earlier default holds it explicitly (⌘⇧H/⌘⇧L up to 0.2.0,
    /// then ⌥⇧H/⌥⇧L): loading it must move both to ⌃⌥, or existing installs would keep
    /// Finder's and Safari's combos, or ones that secure input silences.
    #[test]
    fn loading_old_settings_moves_quick_hide_and_listen_to_the_current_defaults() {
        let dir = std::env::temp_dir().join("cloakview-settings-migrate-test");
        let path = settings_path(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for (hide, listen) in [("cmdOrControl+shift+h", "cmdOrControl+shift+l"), ("alt+shift+h", "alt+shift+l")] {
            std::fs::write(
                &path,
                format!(
                    r#"{{ "language": "es", "shortcuts": {{
                    "quickHide": "{hide}", "pushToAsk": "cmdOrControl+shift+space",
                    "toggleListen": "{listen}", "readScreen": "cmdOrControl+shift+s" }} }}"#
                ),
            )
            .unwrap();
            let s = load_from(&path);
            assert_eq!(s.language, "es"); // the rest of the file is read as before
            assert_eq!(s.shortcuts.quick_hide, "ctrl+alt+h", "from {hide}");
            assert_eq!(s.shortcuts.toggle_listen, "ctrl+alt+l", "from {listen}");
            assert_eq!(s.shortcuts.push_to_ask, "cmdOrControl+shift+space");
            assert_eq!(s.shortcuts.read_screen, "cmdOrControl+shift+s");
        }
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

    #[test]
    fn patch_clamps_reply_delay_ms_to_a_sensible_range() {
        let mut s = Settings::default();
        assert_eq!(s.reply_delay_ms, 1000);

        apply_patch(&mut s, SettingsPatch { reply_delay_ms: Some(2500), ..Default::default() });
        assert_eq!(s.reply_delay_ms, 2500);

        apply_patch(&mut s, SettingsPatch { reply_delay_ms: Some(10_000), ..Default::default() });
        assert_eq!(s.reply_delay_ms, 5000);

        apply_patch(&mut s, SettingsPatch { reply_delay_ms: Some(0), ..Default::default() });
        assert_eq!(s.reply_delay_ms, 0);
    }

    /// A `settings.json` written by the shipped 0.2.1 build (schema at tag
    /// `v0.2.1`: it has `answersModel`, no provider fields) must load
    /// cleanly on the new build: no error, every field it set preserved, new
    /// fields defaulted — and the transcription model actually sent must be
    /// the same one 0.2.1 sent (0.2.1 saved `gpt-4o-mini-realtime`, which
    /// its realtime client already fell back to `gpt-4o-mini-transcribe`).
    #[test]
    fn settings_json_written_by_v021_loads_without_error_or_loss() {
        let dir = std::env::temp_dir().join("cloakview-settings-v021-fixture");
        let path = settings_path(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            &path,
            r#"{
  "language": "es",
  "theme": "light",
  "opacity": 0.85,
  "answersModel": "gpt-4o-mini",
  "realtimeModel": "gpt-4o-mini-realtime",
  "userKeywords": ["opinas", "christian"],
  "userName": "Christian",
  "shortcuts": {
    "quickHide": "ctrl+alt+h",
    "pushToAsk": "cmdOrControl+shift+space",
    "toggleListen": "ctrl+alt+l",
    "readScreen": "cmdOrControl+shift+s"
  },
  "includeMic": true,
  "personalContext": "Rust engineer, interviewing for fintech roles.",
  "answerStyle": "interview",
  "translateTo": "en"
}"#,
        )
        .unwrap();

        let s = load_from(&path);

        // Everything 0.2.1 wrote survives, nothing is silently reset.
        assert_eq!(s.language, "es");
        assert_eq!(s.theme, "light");
        assert!((s.opacity - 0.85).abs() < f64::EPSILON);
        assert_eq!(s.user_keywords, vec!["opinas", "christian"]);
        assert_eq!(s.user_name.as_deref(), Some("Christian"));
        assert_eq!(s.shortcuts, crate::hotkeys::Shortcuts::default());
        assert!(s.include_mic);
        assert_eq!(s.personal_context, "Rust engineer, interviewing for fintech roles.");
        assert_eq!(s.answer_style, "interview");
        assert_eq!(s.translate_to, "en");

        // New fields pick up their defaults; the removed `answersModel` is ignored.
        assert_eq!(s.active_provider, "openai");
        assert!(s.model_overrides.is_empty());
        assert_eq!(s.reply_delay_ms, 1000);

        // 0.2.1's saved value is kept verbatim, and the model the realtime
        // session actually sends is the same one 0.2.1 sent — the new
        // default (`gpt-4o-mini-transcribe`) equals 0.2.1's fallback.
        assert_eq!(s.realtime_model, "gpt-4o-mini-realtime");
        assert_eq!(
            crate::realtime::client::transcription_model(&s.realtime_model),
            "gpt-4o-mini-transcribe",
            "an upgraded install must keep sending the model 0.2.1 sent"
        );
        assert_eq!(
            crate::realtime::client::transcription_model(&Settings::default().realtime_model),
            "gpt-4o-mini-transcribe",
            "the new default must match what 0.2.1 actually sent"
        );
    }
}
