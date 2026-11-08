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
    /// fires (e.g. the user's own name, "¿tú qué opinas?"). Empty by
    /// default; configured by the user in Settings.
    pub user_keywords: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: "en".into(),
            theme: "dark".into(),
            opacity: 0.95,
            answers_model: "gpt-4o-mini".into(),
            realtime_model: "gpt-4o-mini-realtime".into(),
            user_keywords: Vec::new(),
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
