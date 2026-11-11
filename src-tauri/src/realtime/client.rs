use crate::audio::AudioChunk;
use crate::error::{AppError, AppResult};
use crate::realtime::{parse::parse_frame, Backoff, RealtimeEvent};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http;

const URL: &str = "wss://api.openai.com/v1/realtime";

/// Owns the realtime WS session lifecycle: connect, send `session.update`,
/// pump outgoing audio and incoming frames, reconnect with backoff on
/// drop/error. Returns only on explicit stop (audio channel closed by
/// Task 11) or when building the request itself fails.
///
/// Forwards every parsed `RealtimeEvent` through `events_out` instead of
/// emitting to the frontend itself (that moved to `listen::pump`, which
/// maps each variant to its spec-named event — see that module's doc).
/// `try_send`, not `.send().await`: this must never block the WS read
/// loop on a slow/stuck consumer, matching the same drop-and-keep-reading
/// rule `answers::client::stream_answer` (Task 10) already uses.
pub async fn run_session(
    api_key: String,
    model: String,
    mut audio_in: tokio::sync::mpsc::Receiver<AudioChunk>,
    events_out: tokio::sync::mpsc::Sender<RealtimeEvent>,
) -> AppResult<()> {
    let mut backoff = Backoff::default();
    loop {
        let url = format!("{URL}?model={model}");
        let mut req = url
            .into_client_request()
            .map_err(|e| AppError::Network(e.to_string()))?;
        req.headers_mut().insert(
            http::header::AUTHORIZATION,
            format!("Bearer {api_key}")
                .parse()
                .map_err(|e: http::header::InvalidHeaderValue| AppError::Network(e.to_string()))?,
        );
        req.headers_mut().insert(
            "OpenAI-Beta",
            http::header::HeaderValue::from_static("realtime=v1"),
        );

        match connect_async(req).await {
            Ok((ws, _)) => {
                backoff.reset();
                let (mut write, mut read) = ws.split();
                let _ = events_out.try_send(RealtimeEvent::SessionStarted);

                // session.update: VAD turn detection + transcription
                let session_cfg = serde_json::json!({
                    "type": "session.update",
                    "session": {
                        "input_audio_transcription": { "model": "whisper-1" },
                        "turn_detection": { "type": "server_vad" },
                        "input_audio_format": "pcm16",
                    }
                });
                if write.send(session_cfg.to_string().into()).await.is_err() {
                    tokio::time::sleep(backoff.next_delay()).await;
                    continue;
                }

                loop {
                    tokio::select! {
                        chunk = audio_in.recv() => match chunk {
                            Some(c) => {
                                let b64 = base64::Engine::encode(
                                    &base64::engine::general_purpose::STANDARD,
                                    &c.pcm_s16le,
                                );
                                let frame = serde_json::json!({
                                    "type": "input_audio_buffer.append",
                                    "audio": b64,
                                });
                                if write.send(frame.to_string().into()).await.is_err() {
                                    break;
                                }
                            }
                            None => return Ok(()), // explicit stop from Task 11
                        },
                        msg = read.next() => match msg {
                            Some(Ok(tokio_tungstenite::tungstenite::Message::Text(txt))) => {
                                if let Some(ev) = parse_frame(txt.as_str()) {
                                    let stop = matches!(ev, RealtimeEvent::Error { .. });
                                    let _ = events_out.try_send(ev);
                                    if stop {
                                        break;
                                    }
                                }
                            }
                            Some(Ok(_)) => {} // pings/binary ignored
                            _ => break, // stream ended -> reconnect
                        },
                    }
                }
            }
            Err(e) => {
                let _ = events_out.try_send(RealtimeEvent::Error { message: e.to_string() });
            }
        }
        tokio::time::sleep(backoff.next_delay()).await;
    }
}
