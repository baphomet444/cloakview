//! Session orchestrator: wires an `AudioSource` -> `pump` -> the realtime
//! WS client (`realtime::client::run_session`), and tracks the running
//! session so `stop` can cancel it and wait for a clean shutdown.
//!
//! `pump` is also where the spec's five frontend events get their final
//! shape: `realtime::client::run_session` no longer calls `app.emit`
//! itself (see that file) — it only forwards parsed `RealtimeEvent`s
//! through a channel, and `pump` is the single place that maps each one
//! to its spec-named event with a bare, undiscriminated payload:
//!
//! | `RealtimeEvent` variant | frontend event     | payload         |
//! |-------------------------|---------------------|-----------------|
//! | `Transcript { .. }`     | `"transcript-delta"`| the `Turn`      |
//! | `TurnBoundary`          | `"turn-detected"`   | `null`          |
//! | `Error { message }`     | `"error"`           | `message`       |
//! | `SessionStarted`        | `"listen-state"`    | `Connected`     |
//! | `Closed`                | `"listen-state"`    | `Reconnecting`  |
//!
//! `SessionStarted`/`Closed` were explicitly left for me to choose a
//! destination for (not their own event): they're WS-connection-lifecycle
//! detail, not something `AnswersEvent`-style callers need as a distinct
//! channel, so they extend `ListenState` instead of adding a 7th event
//! name. `AudioStatus::Error` also goes through `"listen-state"` as
//! `Error { message }`, unchanged from before.
//!
//! **`"turn-detected"`'s real trigger is not `TurnBoundary`.**
//! `realtime::parse::parse_frame` never actually constructs that variant
//! from a real OpenAI frame (see its own test module), so relying on it
//! would mean this event — the product's core "it's your turn to
//! speak" signal — never fires at all. Instead, every `Transcript` runs
//! through `detect::is_user_turn(&turn.text, user_keywords, turn.is_user_turn)`
//! right after it's buffered and its `"transcript-delta"` is emitted; a
//! `true` result emits `"turn-detected"` too. The `TurnBoundary` match arm
//! stays wired as a harmless, currently-unreachable fallback in case a
//! future `parse_frame` change starts producing it — but it is not what
//! makes the feature work today.
//!
//! **Keywords and the user's name are read once per session.** `start`
//! snapshots `Settings::user_keywords` and `Settings::user_name` when
//! listening begins and `pump` uses that snapshot until it ends; saving new
//! values through `save_settings` while a session is running does **not**
//! affect it. Changing them takes effect only after stopping and starting
//! listening again. (A clean install works with no configuration: the
//! defaults are seeded with the spec's "¿tú qué opinas?" phrase — see
//! `Settings::default`.)
//!
//! `prepare_session`, `pump`, and `pump_until_cancelled` have no `tauri`
//! imports and are unit-tested directly (this crate's module rule: pure
//! logic/orchestration-core is testable, I/O glue stays thin) — `emit` is
//! `&dyn Fn(&str, serde_json::Value)` rather than a `tauri::AppHandle`.
//! `start` *does* take a `tauri::AppHandle` and is exercised by the Task
//! 11 manual checklist, not a unit test — the same split Task 9 already
//! drew around `run_session` itself.

use crate::audio::{AudioChunk, AudioSource, AudioStatus};
use crate::error::AppResult;
use crate::realtime::{self, RealtimeEvent};
use crate::secrets;
use crate::transcript::Turn;
use serde::Serialize;
use tauri::{Emitter, Manager};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

const LISTEN_STATE_EVENT: &str = "listen-state";
const TRANSCRIPT_DELTA_EVENT: &str = "transcript-delta";
const TURN_DETECTED_EVENT: &str = "turn-detected";
const ERROR_EVENT: &str = "error";

/// What `detect::is_user_turn` needs besides the text itself: the keyword
/// list and the user's name, snapshotted once at session start (see the
/// module doc). Bundled so `pump`/`pump_until_cancelled` don't grow a
/// parameter per detection input.
#[derive(Debug, Clone, Copy, Default)]
struct TurnDetection<'a> {
    keywords: &'a [String],
    user_name: Option<&'a str>,
}

/// State surfaced to the frontend under the `"listen-state"` event.
/// `Connected`/`Reconnecting` mirror the realtime WS connection's own
/// lifecycle (`RealtimeEvent::SessionStarted`/`Closed`) — see the module
/// doc for why they land here instead of a separate event.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ListenState {
    Listening,
    Connected,
    Reconnecting,
    Stopped,
    Error { message: String },
}

/// A running listen session. `stop` cancels it, asks the audio source to
/// tear down, and waits for the task to actually finish before returning —
/// callers never race a session that's still shutting down.
pub struct SessionHandle {
    cancel: CancellationToken,
    handle: JoinHandle<()>,
    src: Box<dyn AudioSource>,
    app: tauri::AppHandle,
}

impl SessionHandle {
    pub async fn stop(mut self) {
        self.cancel.cancel();
        let _ = self.src.stop();
        let _ = self.handle.await;
        let _ = self.app.emit(LISTEN_STATE_EVENT, ListenState::Stopped);
    }
}

/// Resolves the OpenAI API key and starts `src` — in that order, and both
/// *before* any task is spawned: `secrets::get_api_key()` surfaces
/// `AppError::MissingApiKey` synchronously, so a missing key is never
/// discovered from inside a spawned task. No `tauri` import — unit-tested
/// directly via `start_without_key_is_missing_api_key`.
fn prepare_session(
    mut src: Box<dyn AudioSource>,
) -> AppResult<(String, Box<dyn AudioSource>, mpsc::Receiver<AudioStatus>)> {
    let api_key = secrets::get_api_key()?;
    let (tx, rx) = mpsc::channel(32);
    src.start(tx)?;
    Ok((api_key, src, rx))
}

/// Appends `turn`, dropping the oldest entry first once at capacity.
async fn push_turn(transcript: &tokio::sync::Mutex<Vec<Turn>>, turn: Turn) {
    let mut buf = transcript.lock().await;
    if buf.len() >= crate::transcript::CAPACITY {
        buf.remove(0);
    }
    buf.push(turn);
}

/// One `AudioStatus` message's effect on the pump loop: forward a chunk,
/// or stop (end of stream / fatal error). Shared by both branches of
/// `pump`'s loop (selecting on `realtime_events` or not) so the handling
/// isn't duplicated.
async fn handle_audio_status(
    status: Option<AudioStatus>,
    audio_out: &mpsc::Sender<AudioChunk>,
    emit: &(dyn Fn(&str, serde_json::Value) + Send + Sync),
) -> std::ops::ControlFlow<()> {
    match status {
        Some(AudioStatus::Chunk(c)) => {
            if audio_out.send(c).await.is_err() {
                return std::ops::ControlFlow::Break(()); // realtime session is gone.
            }
            std::ops::ControlFlow::Continue(())
        }
        Some(AudioStatus::Ended) | None => std::ops::ControlFlow::Break(()),
        Some(AudioStatus::Error(message)) => {
            emit(
                LISTEN_STATE_EVENT,
                serde_json::to_value(ListenState::Error { message }).expect("ListenState serializes"),
            );
            std::ops::ControlFlow::Break(())
        }
    }
}

/// Forwards audio chunks from `audio_status` to `audio_out` (consumed by
/// `realtime::client::run_session`), and buffers/re-emits `realtime_events`
/// per the table in the module doc. `AudioStatus::Error` emits exactly one
/// `"listen-state"` error and returns — it never loops back to read again.
/// `emit` is `&dyn Fn` rather than a `tauri::AppHandle` so this is
/// testable without Tauri.
async fn pump(
    mut audio_status: mpsc::Receiver<AudioStatus>,
    audio_out: mpsc::Sender<AudioChunk>,
    mut realtime_events: mpsc::Receiver<RealtimeEvent>,
    transcript: &tokio::sync::Mutex<Vec<Turn>>,
    detection: TurnDetection<'_>,
    emit: &(dyn Fn(&str, serde_json::Value) + Send + Sync),
) {
    let mut realtime_open = true;
    loop {
        if realtime_open {
            tokio::select! {
                status = audio_status.recv() => {
                    if handle_audio_status(status, &audio_out, emit).await.is_break() {
                        return;
                    }
                }
                ev = realtime_events.recv() => match ev {
                    Some(RealtimeEvent::Transcript { speaker, text, is_user_turn }) => {
                        let turn = Turn { speaker, text, is_user_turn, ts: crate::db::now_ms() };
                        push_turn(transcript, turn.clone()).await;
                        emit(
                            TRANSCRIPT_DELTA_EVENT,
                            serde_json::to_value(&turn).expect("Turn serializes"),
                        );
                        if crate::detect::is_user_turn(
                            &turn.text,
                            detection.keywords,
                            detection.user_name,
                            turn.is_user_turn,
                        ) {
                            emit(TURN_DETECTED_EVENT, serde_json::Value::Null);
                        }
                    }
                    // Unreachable today (see module doc) — kept so a future
                    // `parse_frame` producing this doesn't silently need
                    // new wiring here too.
                    Some(RealtimeEvent::TurnBoundary) => emit(TURN_DETECTED_EVENT, serde_json::Value::Null),
                    Some(RealtimeEvent::Error { message }) => {
                        emit(ERROR_EVENT, serde_json::Value::String(message));
                    }
                    Some(RealtimeEvent::SessionStarted) => emit(
                        LISTEN_STATE_EVENT,
                        serde_json::to_value(ListenState::Connected).expect("ListenState serializes"),
                    ),
                    Some(RealtimeEvent::Closed) => emit(
                        LISTEN_STATE_EVENT,
                        serde_json::to_value(ListenState::Reconnecting).expect("ListenState serializes"),
                    ),
                    None => realtime_open = false, // run_session ended; keep pumping audio until it also closes.
                },
            }
        } else {
            let status = audio_status.recv().await;
            if handle_audio_status(status, &audio_out, emit).await.is_break() {
                return;
            }
        }
    }
}

/// Runs `pump` under `cancel`: returns as soon as either `cancel` fires or
/// `pump` itself returns. This is the exact mechanism `SessionHandle::stop`
/// depends on — it calls `cancel.cancel()` then awaits the `JoinHandle`
/// wrapping a call built on this, so cancellation is observable as prompt
/// task completion.
async fn pump_until_cancelled(
    cancel: CancellationToken,
    status_rx: mpsc::Receiver<AudioStatus>,
    audio_out: mpsc::Sender<AudioChunk>,
    realtime_events: mpsc::Receiver<RealtimeEvent>,
    transcript: &tokio::sync::Mutex<Vec<Turn>>,
    detection: TurnDetection<'_>,
    emit: &(dyn Fn(&str, serde_json::Value) + Send + Sync),
) {
    tokio::select! {
        _ = cancel.cancelled() => {}
        _ = pump(status_rx, audio_out, realtime_events, transcript, detection, emit) => {}
    }
}

/// Starts a listen session: resolves the key and starts the real
/// `SckAudioSource` synchronously (see `prepare_session`), then spawns one
/// task that pumps audio into `realtime::client::run_session` (and its
/// parsed events back out) until cancelled, a fatal audio error, or the
/// realtime session itself returns.
pub fn start(model: String, app: tauri::AppHandle) -> AppResult<SessionHandle> {
    let src: Box<dyn AudioSource> = Box::new(crate::audio::screencapturekit::SckAudioSource::new()?);
    let (api_key, src, status_rx) = prepare_session(src)?;

    let (audio_tx, audio_rx) = mpsc::channel::<AudioChunk>(32);
    let (rt_tx, rt_rx) = mpsc::channel::<RealtimeEvent>(64);
    let cancel = CancellationToken::new();
    let cancel_for_task = cancel.clone();

    let app_for_handle = app.clone();
    let listen_state_app = app.clone();
    let generic_app = app.clone();

    let emit_listen_state = move |state: ListenState| {
        let _ = listen_state_app.emit(LISTEN_STATE_EVENT, state);
    };
    let emit_generic = move |event: &str, payload: serde_json::Value| {
        let _ = generic_app.emit(event, payload);
    };

    let handle = tokio::spawn(async move {
        emit_listen_state(ListenState::Listening);
        let app_state = app.state::<crate::state::AppState>();
        // Snapshot, once per session: later `save_settings` calls do not
        // reach a running session (documented in the module doc).
        let (user_keywords, user_name) = {
            let s = app_state.settings.read().expect("settings lock");
            (s.user_keywords.clone(), s.user_name.clone())
        };
        let detection = TurnDetection { keywords: &user_keywords, user_name: user_name.as_deref() };
        let realtime_task = realtime::client::run_session(api_key, model, audio_rx, rt_tx);
        tokio::select! {
            _ = pump_until_cancelled(
                cancel_for_task, status_rx, audio_tx, rt_rx, &app_state.transcript, detection,
                &emit_generic,
            ) => {}
            res = realtime_task => {
                if let Err(e) = res {
                    emit_listen_state(ListenState::Error { message: e.to_string() });
                }
            }
        }
    });

    Ok(SessionHandle { cancel, handle, src, app: app_for_handle })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::FakeAudioSource;
    use crate::error::AppError;

    /// Every `(event name, payload)` a test's `emit` was called with.
    type Recorded = std::sync::Arc<std::sync::Mutex<Vec<(String, serde_json::Value)>>>;

    fn recording_emit() -> (impl Fn(&str, serde_json::Value) + Send + Sync, Recorded) {
        let emitted = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let recorder = emitted.clone();
        let emit = move |name: &str, payload: serde_json::Value| {
            recorder.lock().unwrap().push((name.to_string(), payload));
        };
        (emit, emitted)
    }

    /// Runs `pump` over a single remote `Transcript` with the given
    /// detection inputs and returns the names of the events it emitted.
    async fn emitted_names_for(text: &str, detection: TurnDetection<'_>) -> Vec<String> {
        let (_status_tx, status_rx) = mpsc::channel::<AudioStatus>(1);
        let (audio_tx, _audio_rx) = mpsc::channel::<AudioChunk>(1);
        let (rt_tx, rt_rx) = mpsc::channel::<RealtimeEvent>(1);
        let transcript = tokio::sync::Mutex::new(Vec::<Turn>::new());
        let (emit, emitted) = recording_emit();

        rt_tx
            .send(RealtimeEvent::Transcript {
                speaker: "remote".into(),
                text: text.into(),
                is_user_turn: false,
            })
            .await
            .unwrap();

        // `pump` never returns on its own here; the timeout just bounds the wait
        // (the queued `Transcript` is handled on its first scheduling).
        let _ = tokio::time::timeout(
            std::time::Duration::from_millis(200),
            pump(status_rx, audio_tx, rt_rx, &transcript, detection, &emit),
        )
        .await;

        let events = emitted.lock().unwrap();
        let names: Vec<String> = events.iter().map(|(n, _)| n.clone()).collect();
        names
    }

    #[tokio::test]
    async fn clean_install_defaults_fire_turn_detected_without_any_configuration() {
        // Exactly what `start` would snapshot on a fresh install: nothing
        // configured, so only `Settings::default()` is in play.
        let settings = crate::state::Settings::default();
        let detection = TurnDetection {
            keywords: &settings.user_keywords,
            user_name: settings.user_name.as_deref(),
        };

        // The spec's phrase, with and without the Spanish punctuation
        // (Whisper often drops the ¿?), and capitalized like a sentence.
        for text in ["¿Tú qué opinas?", "Tú qué opinas", "y bien, tú qué opinas de esto"] {
            assert_eq!(
                emitted_names_for(text, detection).await,
                vec![TRANSCRIPT_DELTA_EVENT, TURN_DETECTED_EVENT],
                "default keywords should detect {text:?}"
            );
        }

        // ...and the defaults are not a catch-all.
        assert_eq!(
            emitted_names_for("buenos días a todos", detection).await,
            vec![TRANSCRIPT_DELTA_EVENT]
        );
    }

    #[tokio::test]
    async fn user_name_fires_turn_detected_and_none_does_not_break() {
        let named = TurnDetection { keywords: &[], user_name: Some("Christian") };
        assert_eq!(
            emitted_names_for("Christian, ¿puedes explicar?", named).await,
            vec![TRANSCRIPT_DELTA_EVENT, TURN_DETECTED_EVENT]
        );

        // No name configured (and no keywords): the same line is just a delta.
        let anonymous = TurnDetection { keywords: &[], user_name: None };
        assert_eq!(
            emitted_names_for("Christian, ¿puedes explicar?", anonymous).await,
            vec![TRANSCRIPT_DELTA_EVENT]
        );
    }

    #[tokio::test]
    async fn start_without_key_is_missing_api_key() {
        let _guard = secrets::TEST_LOCK.lock().unwrap();
        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
        let _ = secrets::clear_api_key();

        let src: Box<dyn AudioSource> =
            Box::new(FakeAudioSource { chunks: vec![], fail_after: None });

        let result = prepare_session(src);
        assert!(matches!(result, Err(AppError::MissingApiKey)));
    }

    #[tokio::test]
    async fn audio_error_emits_error_state() {
        let mut src = FakeAudioSource {
            chunks: vec![AudioChunk { pcm_s16le: vec![0; 4], sample_rate: 16_000, channels: 1 }],
            fail_after: Some(0),
        };
        let (status_tx, status_rx) = mpsc::channel(8);
        src.start(status_tx).unwrap();

        let (audio_tx, _audio_rx) = mpsc::channel::<AudioChunk>(8);
        let (_rt_tx, rt_rx) = mpsc::channel::<RealtimeEvent>(1);
        let transcript = tokio::sync::Mutex::new(Vec::<Turn>::new());
        let (emit, emitted) = recording_emit();

        pump(status_rx, audio_tx, rt_rx, &transcript, TurnDetection::default(), &emit).await;

        let events = emitted.lock().unwrap();
        assert_eq!(events.len(), 1, "expected exactly one emission, got {events:?}");
        assert_eq!(events[0].0, LISTEN_STATE_EVENT);
        assert_eq!(events[0].1["kind"], "error");
    }

    #[tokio::test]
    async fn pump_emits_transcript_delta_with_turn_payload() {
        let (_status_tx, status_rx) = mpsc::channel::<AudioStatus>(1);
        let (audio_tx, _audio_rx) = mpsc::channel::<AudioChunk>(1);
        let (rt_tx, rt_rx) = mpsc::channel::<RealtimeEvent>(1);
        let transcript = tokio::sync::Mutex::new(Vec::<Turn>::new());
        let (emit, emitted) = recording_emit();

        rt_tx
            .send(RealtimeEvent::Transcript {
                speaker: "remote".into(),
                text: "hola".into(),
                is_user_turn: false,
            })
            .await
            .unwrap();

        // `pump` never returns on its own (audio stays open, nothing
        // closes it) — the timeout just bounds the wait; the one queued
        // `Transcript` is drained and its emission recorded on `pump`'s
        // very first scheduling, before anything can block for real.
        let _ = tokio::time::timeout(
            std::time::Duration::from_millis(200),
            pump(status_rx, audio_tx, rt_rx, &transcript, TurnDetection::default(), &emit),
        )
        .await;

        let events = emitted.lock().unwrap();
        assert_eq!(events.len(), 1, "expected exactly one emission, got {events:?}");
        assert_eq!(events[0].0, TRANSCRIPT_DELTA_EVENT);
        assert_eq!(events[0].1["speaker"], "remote");
        assert_eq!(events[0].1["text"], "hola");
        assert_eq!(events[0].1["isUserTurn"], false);
        assert!(events[0].1["ts"].is_i64());
    }

    #[tokio::test]
    async fn pump_buffers_transcript_turns_in_order() {
        let (_status_tx, status_rx) = mpsc::channel::<AudioStatus>(1);
        let (audio_tx, _audio_rx) = mpsc::channel::<AudioChunk>(1);
        let (rt_tx, rt_rx) = mpsc::channel::<RealtimeEvent>(4);
        let transcript = tokio::sync::Mutex::new(Vec::<Turn>::new());

        rt_tx
            .send(RealtimeEvent::Transcript { speaker: "a".into(), text: "one".into(), is_user_turn: false })
            .await
            .unwrap();
        rt_tx
            .send(RealtimeEvent::Transcript { speaker: "b".into(), text: "two".into(), is_user_turn: true })
            .await
            .unwrap();

        let _ = tokio::time::timeout(
            std::time::Duration::from_millis(200),
            pump(status_rx, audio_tx, rt_rx, &transcript, TurnDetection::default(), &|_, _| {}),
        )
        .await;

        let buf = transcript.lock().await;
        assert_eq!(buf.len(), 2, "expected two buffered turns, got {buf:?}");
        assert_eq!(buf[0].text, "one");
        assert_eq!(buf[1].text, "two");
    }

    #[tokio::test]
    async fn stop_cancels_the_pump() {
        let (_status_tx, status_rx) = mpsc::channel::<AudioStatus>(8);
        let (audio_tx, _audio_rx) = mpsc::channel::<AudioChunk>(8);
        let (_rt_tx, rt_rx) = mpsc::channel::<RealtimeEvent>(8);
        let transcript = tokio::sync::Mutex::new(Vec::<Turn>::new());
        let cancel = CancellationToken::new();
        let cancel_child = cancel.clone();

        cancel.cancel();

        tokio::time::timeout(
            std::time::Duration::from_secs(1),
            pump_until_cancelled(
                cancel_child,
                status_rx,
                audio_tx,
                rt_rx,
                &transcript,
                TurnDetection::default(),
                &|_, _| {},
            ),
        )
        .await
        .expect("cancel() should make pump_until_cancelled return promptly");
    }

    #[tokio::test]
    async fn pump_emits_turn_detected_only_when_a_keyword_matches() {
        let keywords = vec!["opinas".to_string()];
        let detection = TurnDetection { keywords: &keywords, user_name: None };

        // Contains a configured keyword: transcript-delta AND turn-detected.
        {
            let (_status_tx, status_rx) = mpsc::channel::<AudioStatus>(1);
            let (audio_tx, _audio_rx) = mpsc::channel::<AudioChunk>(1);
            let (rt_tx, rt_rx) = mpsc::channel::<RealtimeEvent>(1);
            let transcript = tokio::sync::Mutex::new(Vec::<Turn>::new());
            let (emit, emitted) = recording_emit();

            rt_tx
                .send(RealtimeEvent::Transcript {
                    speaker: "remote".into(),
                    text: "¿tú qué opinas?".into(),
                    is_user_turn: false,
                })
                .await
                .unwrap();

            let _ = tokio::time::timeout(
                std::time::Duration::from_millis(200),
                pump(status_rx, audio_tx, rt_rx, &transcript, detection, &emit),
            )
            .await;

            let events = emitted.lock().unwrap();
            let names: Vec<&str> = events.iter().map(|(n, _)| n.as_str()).collect();
            assert_eq!(
                names,
                vec![TRANSCRIPT_DELTA_EVENT, TURN_DETECTED_EVENT],
                "keyword match should emit both events, got {events:?}"
            );
        }

        // No keyword match: transcript-delta only.
        {
            let (_status_tx, status_rx) = mpsc::channel::<AudioStatus>(1);
            let (audio_tx, _audio_rx) = mpsc::channel::<AudioChunk>(1);
            let (rt_tx, rt_rx) = mpsc::channel::<RealtimeEvent>(1);
            let transcript = tokio::sync::Mutex::new(Vec::<Turn>::new());
            let (emit, emitted) = recording_emit();

            rt_tx
                .send(RealtimeEvent::Transcript {
                    speaker: "remote".into(),
                    text: "buenos días a todos".into(),
                    is_user_turn: false,
                })
                .await
                .unwrap();

            let _ = tokio::time::timeout(
                std::time::Duration::from_millis(200),
                pump(status_rx, audio_tx, rt_rx, &transcript, detection, &emit),
            )
            .await;

            let events = emitted.lock().unwrap();
            let names: Vec<&str> = events.iter().map(|(n, _)| n.as_str()).collect();
            assert_eq!(
                names,
                vec![TRANSCRIPT_DELTA_EVENT],
                "no keyword match should emit transcript-delta only, got {events:?}"
            );
        }
    }

    #[test]
    fn listen_state_serializes_with_camel_case_kind() {
        assert_eq!(
            serde_json::to_value(ListenState::Listening).unwrap(),
            serde_json::json!({"kind": "listening"})
        );
        assert_eq!(
            serde_json::to_value(ListenState::Connected).unwrap(),
            serde_json::json!({"kind": "connected"})
        );
        assert_eq!(
            serde_json::to_value(ListenState::Reconnecting).unwrap(),
            serde_json::json!({"kind": "reconnecting"})
        );
        assert_eq!(
            serde_json::to_value(ListenState::Stopped).unwrap(),
            serde_json::json!({"kind": "stopped"})
        );
        assert_eq!(
            serde_json::to_value(ListenState::Error { message: "boom".into() }).unwrap(),
            serde_json::json!({"kind": "error", "message": "boom"})
        );
    }
}
