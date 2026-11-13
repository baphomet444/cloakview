use std::sync::{Mutex, RwLock};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub language: String,
    pub theme: String,
    pub opacity: f64,
    pub answers_model: String,
    pub realtime_model: String,
    /// Keywords `listen::pump` checks against each `Transcript`'s text —
    /// via `detect::is_user_turn` — to decide whether `"turn-detected"`
    /// fires. Seeded with the spec's example phrase so a clean install
    /// detects turns without any configuration. Read once per listen
    /// session (see `listen.rs`).
    pub user_keywords: Vec<String>,
    /// The user's own name, matched by `detect::is_user_turn` as one more
    /// keyword when set. `None` until the user configures it. Read once
    /// per listen session (see `listen.rs`).
    pub user_name: Option<String>,
    /// Global shortcut accelerators (`hotkeys`). Re-registered on every save that
    /// changes them.
    pub shortcuts: crate::hotkeys::Shortcuts,
    /// Also transcribe the user's microphone. Off by default: without headphones the
    /// microphone hears the speakers. Read when listening starts.
    pub include_mic: bool,
    /// The user's own notes about themselves (résumé, job description, product
    /// docs), appended to the answer instructions so suggestions are tailored to
    /// them. CloakView Pro: ignored without a license. Goes to OpenAI with each
    /// answer, like everything else.
    pub personal_context: String,
    /// How the answer should be shaped: `balanced` (the default), `interview`
    /// (STAR), `sales`, `technical` or `short`. CloakView Pro: ignored without
    /// a license.
    pub answer_style: String,
    /// Live translation target for the transcript: "" (off), "en" or "es".
    /// CloakView Pro: ignored without a license.
    pub translate_to: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: "en".into(),
            theme: "dark".into(),
            opacity: 0.95,
            answers_model: "gpt-4o-mini".into(),
            realtime_model: "gpt-4o-mini-realtime".into(),
            // The spec's example phrase (design.md §5, `detect`) and its
            // punctuation-free variant — Whisper frequently drops the ¿?.
            // Lowercase because `detect` compares case-insensitively anyway.
            user_keywords: vec!["¿tú qué opinas?".into(), "tú qué opinas".into()],
            user_name: None,
            shortcuts: crate::hotkeys::Shortcuts::default(),
            include_mic: false,
            personal_context: String::new(),
            answer_style: "balanced".into(),
            translate_to: String::new(),
        }
    }
}

#[derive(Default)]
pub struct AppState {
    pub settings: RwLock<Settings>,
    pub db: Mutex<Option<crate::db::Db>>,
    /// Recent transcript turns (bounded to `transcript::CAPACITY`), used as
    /// context for `request_answer`. Fed by `listen::pump` as
    /// `RealtimeEvent::Transcript` events arrive. `tokio::sync::Mutex`, not
    /// `std::sync::Mutex`: `pump` is async and holds this lock across
    /// `.await` points (see `listen.rs`), which a std guard can't do
    /// without risking a blocked executor thread.
    pub transcript: tokio::sync::Mutex<Vec<crate::transcript::Turn>>,
    /// The running listen session, if any. `tokio::sync::Mutex` (not
    /// `std::sync::Mutex`): `start_listen`/`stop_listen` hold this lock
    /// across `.await` points (spawning/awaiting the session task), which
    /// a std mutex guard may not be held across.
    pub listen_session: tokio::sync::Mutex<Option<crate::listen::SessionHandle>>,
    /// The in-flight `answers::client::stream_answer` task, if any, so
    /// `stop_answer` can abort it.
    pub answer_task: tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
    /// The interview being recorded (std mutex: only ever held briefly, never across an await).
    pub session_log: Mutex<crate::session::SessionLog>,
    /// The interview that just ended, kept until the next one so it can be saved to a file.
    pub last_session: Mutex<Option<crate::session::FinishedSession>>,
    /// The CloakView Pro license, loaded from `license.json` at startup.
    pub license: RwLock<crate::license::License>,
    /// The accelerators the last `hotkeys::apply` could not register, so the hub can
    /// show failures that happened before its page was listening (at startup).
    pub shortcut_failures: Mutex<Vec<String>>,
}

impl AppState {
    /// Whether the Pro features are unlocked (`license`). Every Pro gate in
    /// `commands` asks this.
    pub fn is_pro(&self) -> bool {
        self.license.read().expect("license lock").is_pro()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_seeds_the_spec_keywords_in_lowercase() {
        let s = Settings::default();
        assert_eq!(s.user_keywords, vec!["¿tú qué opinas?", "tú qué opinas"]);
        assert!(s.user_keywords.iter().all(|k| *k == k.to_lowercase()));
        assert_eq!(s.user_name, None);
    }

    /// An existing `settings.json` written before these fields existed must
    /// pick up the seeded defaults, not an empty list.
    #[test]
    fn old_settings_json_without_the_new_fields_gets_the_seeded_defaults() {
        let s: Settings = serde_json::from_str(r#"{"language":"es"}"#).unwrap();
        assert_eq!(s.language, "es");
        assert!(!s.user_keywords.is_empty());
        assert_eq!(s.user_name, None);
        assert_eq!(s.shortcuts, crate::hotkeys::Shortcuts::default());
    }

    /// `Settings` goes to the frontend (`get_settings`, `save_settings`) and is persisted
    /// to `settings.json`, so these exact camelCase keys are a contract with the
    /// frontend's `Settings` type. Without `rename_all` it would silently emit
    /// `answers_model`, `realtime_model`, `user_keywords` and `user_name` instead.
    #[test]
    fn settings_serializes_with_camel_case_keys() {
        let s = Settings {
            language: "es".into(),
            theme: "light".into(),
            opacity: 0.5,
            answers_model: "gpt-4o-mini".into(),
            realtime_model: "gpt-4o-mini-realtime".into(),
            user_keywords: vec!["opinas".into()],
            user_name: Some("Ana".into()),
            shortcuts: crate::hotkeys::Shortcuts::default(),
            include_mic: true,
            personal_context: String::new(),
            answer_style: "balanced".into(),
            translate_to: String::new(),
        };
        let json = serde_json::to_value(&s).unwrap();

        assert_eq!(
            json,
            serde_json::json!({
                "language": "es",
                "theme": "light",
                "opacity": 0.5,
                "answersModel": "gpt-4o-mini",
                "realtimeModel": "gpt-4o-mini-realtime",
                "userKeywords": ["opinas"],
                "userName": "Ana",
                "shortcuts": {
                    "quickHide": "ctrl+alt+h",
                    "pushToAsk": "cmdOrControl+shift+space",
                    "toggleListen": "ctrl+alt+l",
                    "readScreen": "cmdOrControl+shift+s"
                },
                "includeMic": true,
                "personalContext": "",
                "answerStyle": "balanced",
                "translateTo": ""
            })
        );
        for key in json.as_object().unwrap().keys() {
            assert!(!key.contains('_'), "snake_case key leaked into the Settings JSON: {key}");
        }
    }

    /// An unset name is sent as `null` (the frontend's type is `string | null`), not omitted.
    #[test]
    fn settings_sends_an_unset_user_name_as_null_not_omitted() {
        let json = serde_json::to_value(Settings::default()).unwrap();
        assert_eq!(json.get("userName"), Some(&serde_json::Value::Null));
    }
}
