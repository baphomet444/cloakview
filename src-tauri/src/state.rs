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
}

impl AppState {
    pub fn is_pro() -> bool {
        false
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
    }
}
