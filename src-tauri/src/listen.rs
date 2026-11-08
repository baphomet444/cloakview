//! Session orchestrator: wires an `AudioSource` -> `pump` -> the realtime
//! WS client (`realtime::client::run_session`), and tracks the running
//! session so `stop` can cancel it and wait for a clean shutdown.
//!
//! `prepare_session`, `pump`, and `pump_until_cancelled` have no `tauri`
//! imports and are unit-tested directly (this crate's module rule: pure
//! logic/orchestration-core is testable, I/O glue stays thin). `start`
//! *does* take a `tauri::AppHandle` (to emit `"listen-state"` and to call
//! `realtime::client::run_session`, which already requires one) and is
//! exercised by the Task 11 manual checklist, not a unit test — the same
//! split Task 9 already drew around `run_session` itself.

use crate::audio::{AudioChunk, AudioSource, AudioStatus};
use crate::error::AppResult;
use crate::realtime;
use crate::secrets;
use serde::Serialize;
use tauri::Emitter;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

const LISTEN_STATE_EVENT: &str = "listen-state";

/// State surfaced to the frontend under the `"listen-state"` event.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ListenState {
    Listening,
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

/// Forwards audio chunks from `audio_status` to `audio_out` (consumed by
/// `realtime::client::run_session`). `AudioStatus::Error` emits exactly one
/// `ListenState::Error` and returns — it never loops back to read again.
/// `emit` is `&dyn Fn` rather than a `tauri::AppHandle` so this is
/// testable without Tauri.
async fn pump(
    mut audio_status: mpsc::Receiver<AudioStatus>,
    audio_out: mpsc::Sender<AudioChunk>,
    emit: &(dyn Fn(ListenState) + Send + Sync),
) {
    while let Some(status) = audio_status.recv().await {
        match status {
            AudioStatus::Chunk(c) => {
                if audio_out.send(c).await.is_err() {
                    return; // realtime session is gone; nothing left to pump into.
                }
            }
            AudioStatus::Ended => return,
            AudioStatus::Error(message) => {
                emit(ListenState::Error { message });
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
    emit: &(dyn Fn(ListenState) + Send + Sync),
) {
    tokio::select! {
        _ = cancel.cancelled() => {}
        _ = pump(status_rx, audio_out, emit) => {}
    }
}

/// Starts a listen session: resolves the key and starts the real
/// `SckAudioSource` synchronously (see `prepare_session`), then spawns one
/// task that pumps audio into `realtime::client::run_session` until
/// cancelled, a fatal audio error, or the realtime session itself returns.
pub fn start(model: String, app: tauri::AppHandle) -> AppResult<SessionHandle> {
    let src: Box<dyn AudioSource> = Box::new(crate::audio::screencapturekit::SckAudioSource::new()?);
    let (api_key, src, status_rx) = prepare_session(src)?;

    let (audio_tx, audio_rx) = mpsc::channel::<AudioChunk>(32);
    let cancel = CancellationToken::new();
    let cancel_for_task = cancel.clone();

    let emit_app = app.clone();
    let app_for_handle = app.clone();
    let emit = move |state: ListenState| {
        let _ = emit_app.emit(LISTEN_STATE_EVENT, state);
    };

    let handle = tokio::spawn(async move {
        emit(ListenState::Listening);
        let realtime_task = realtime::client::run_session(api_key, model, audio_rx, app);
        tokio::select! {
            _ = pump_until_cancelled(cancel_for_task, status_rx, audio_tx, &emit) => {}
            res = realtime_task => {
                if let Err(e) = res {
                    emit(ListenState::Error { message: e.to_string() });
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
        let emitted = std::sync::Arc::new(std::sync::Mutex::new(Vec::<ListenState>::new()));
        let recorder = emitted.clone();
        let emit = move |s: ListenState| recorder.lock().unwrap().push(s);

        pump(status_rx, audio_tx, &emit).await;

        let states = emitted.lock().unwrap();
        assert_eq!(states.len(), 1, "expected exactly one ListenState emission, got {states:?}");
        assert!(matches!(states[0], ListenState::Error { .. }));
    }

    #[tokio::test]
    async fn stop_cancels_the_pump() {
        let (_status_tx, status_rx) = mpsc::channel::<AudioStatus>(8);
        let (audio_tx, _audio_rx) = mpsc::channel::<AudioChunk>(8);
        let cancel = CancellationToken::new();
        let cancel_for_task = cancel.clone();

        let handle = tokio::spawn(async move {
            pump_until_cancelled(cancel_for_task, status_rx, audio_tx, &|_| {}).await;
        });

        // `_status_tx` stays alive and silent: without the cancellation
        // branch, `pump`'s `recv()` would block forever.
        cancel.cancel();

        tokio::time::timeout(std::time::Duration::from_secs(1), handle)
            .await
            .expect("cancel() should make the task join promptly")
            .expect("pump task must not panic");
    }

    #[test]
    fn listen_state_serializes_with_camel_case_kind() {
        assert_eq!(
            serde_json::to_value(ListenState::Listening).unwrap(),
            serde_json::json!({"kind": "listening"})
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
