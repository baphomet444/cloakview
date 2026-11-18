use crate::audio::vad::{TurnEnd, Vad};
use crate::audio::{AudioChunk, TARGET_SAMPLE_RATE};
use crate::error::{AppError, AppResult};
use crate::realtime::{parse::parse_frame, Backoff, RealtimeEvent};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http;

/// Whose audio a session transcribes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// The other participants, from the system audio.
    Remote,
    /// The user, from the microphone.
    Mine,
}

impl Source {
    /// Loudness that counts as speech. A microphone has a noise floor that digital
    /// system audio does not, so it needs a higher bar.
    fn vad(self) -> Vad {
        match self {
            Source::Remote => Vad::default(),
            Source::Mine => Vad::with_threshold(500.0),
        }
    }

    /// OpenAI's noise reduction, applied to the audio before it is transcribed.
    /// The call arrives as the other side's close-talking voice; the microphone is
    /// most often the Mac's own, across the desk.
    fn noise_reduction(self) -> &'static str {
        match self {
            Source::Remote => "near_field",
            Source::Mine => "far_field",
        }
    }

    /// The microphone's lines are the user's: they become `user` turns in the answer
    /// context, and are never a cue to suggest a reply.
    fn label(self, event: RealtimeEvent) -> RealtimeEvent {
        match (self, event) {
            (Source::Mine, RealtimeEvent::Transcript { text, .. }) => {
                RealtimeEvent::Transcript { speaker: "you".into(), text, is_user_turn: true }
            }
            (_, event) => event,
        }
    }
}

/// A transcription session: text only, no spoken reply. The model is chosen in
/// `session.update`, not in the URL (a `?model=` URL opens a conversation
/// session, which rejects the transcription configuration).
const URL: &str = "wss://api.openai.com/v1/realtime?intent=transcription";
/// Transcribes each turn once it is committed. `gpt-live-transcribe` streams
/// text sooner, but accounts on the lower usage tiers are refused by its rate
/// limit before any audio is sent; this model works on them.
const DEFAULT_TRANSCRIPTION_MODEL: &str = "gpt-4o-mini-transcribe";

/// The model for the session. Settings saved by earlier versions name a
/// conversation model (`gpt-4o-mini-realtime`), which a transcription session
/// cannot use, so anything that is not a transcription model falls back to the default.
fn transcription_model(setting: &str) -> &str {
    let setting = setting.trim();
    if setting.contains("transcribe") {
        setting
    } else {
        DEFAULT_TRANSCRIPTION_MODEL
    }
}

/// What a session transcribes with: the model, and the language spoken on the call.
#[derive(Debug, Clone, Default)]
pub struct Transcription {
    pub model: String,
    /// `Settings::call_language`: "en" or "es", or "" to let the model detect it.
    pub language: String,
}

/// The call language as an ISO-639-1 code, when it is one this app offers.
/// Anything else (blank, unknown) means "detect", rather than a session the API refuses.
fn language_code(setting: &str) -> Option<&'static str> {
    match setting.trim() {
        "en" => Some("en"),
        "es" => Some("es"),
        _ => None,
    }
}

fn session_update(transcription: &Transcription, source: Source) -> serde_json::Value {
    let model = transcription_model(&transcription.model);
    let mut config = serde_json::json!({ "model": model });
    // A known language keeps background noise from being "heard" as sentences in
    // other languages. `gpt-live-transcribe` takes a list; the others one code.
    if let Some(code) = language_code(&transcription.language) {
        if model.contains("live-transcribe") {
            config["languages"] = serde_json::json!([code]);
        } else {
            config["language"] = serde_json::json!(code);
        }
    }
    serde_json::json!({
        "type": "session.update",
        "session": {
            "type": "transcription",
            "audio": {
                "input": {
                    "format": { "type": "audio/pcm", "rate": TARGET_SAMPLE_RATE },
                    "noise_reduction": { "type": source.noise_reduction() },
                    "transcription": config,
                    // These models have no server-side turn detection: `Vad` decides.
                    "turn_detection": null,
                }
            }
        }
    })
}

/// Owns the realtime WS session lifecycle: connect, send `session.update`,
/// stream the audio of each spoken turn (see `audio::vad`) and commit it when
/// the speaker pauses, read incoming frames, reconnect with backoff on
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
    transcription: Transcription,
    mut audio_in: tokio::sync::mpsc::Receiver<AudioChunk>,
    events_out: tokio::sync::mpsc::Sender<RealtimeEvent>,
    source: Source,
) -> AppResult<()> {
    let mut backoff = Backoff::default();
    loop {
        let mut req = URL
            .into_client_request()
            .map_err(|e| AppError::Network(e.to_string()))?;
        req.headers_mut().insert(
            http::header::AUTHORIZATION,
            format!("Bearer {api_key}")
                .parse()
                .map_err(|e: http::header::InvalidHeaderValue| AppError::Network(e.to_string()))?,
        );

        match connect_async(req).await {
            Ok((ws, _)) => {
                // Not `backoff.reset()` yet: a session the API refuses (rate limit,
                // unknown model) connects fine and fails right after, and resetting
                // here would retry it every second. A transcript proves it healthy.
                let (mut write, mut read) = ws.split();
                let _ = events_out.try_send(RealtimeEvent::SessionStarted);

                if write.send(session_update(&transcription, source).to_string().into()).await.is_err() {
                    tokio::time::sleep(backoff.next_delay()).await;
                    continue;
                }

                // A new connection has an empty buffer, so turn tracking starts over too.
                let mut vad = source.vad();
                'session: loop {
                    tokio::select! {
                        chunk = audio_in.recv() => match chunk {
                            Some(c) => {
                                let turn = vad.push(c);
                                for chunk in turn.send {
                                    let b64 = base64::Engine::encode(
                                        &base64::engine::general_purpose::STANDARD,
                                        &chunk.pcm_s16le,
                                    );
                                    let frame = serde_json::json!({
                                        "type": "input_audio_buffer.append",
                                        "audio": b64,
                                    });
                                    if write.send(frame.to_string().into()).await.is_err() {
                                        break 'session;
                                    }
                                }
                                let end = match turn.end {
                                    TurnEnd::None => None,
                                    TurnEnd::Commit => Some("input_audio_buffer.commit"),
                                    TurnEnd::Discard => Some("input_audio_buffer.clear"),
                                };
                                if let Some(kind) = end {
                                    crate::diag::log(kind);
                                    let frame = serde_json::json!({ "type": kind });
                                    if write.send(frame.to_string().into()).await.is_err() {
                                        break;
                                    }
                                }
                            }
                            None => return Ok(()), // explicit stop from Task 11
                        },
                        msg = read.next() => match msg {
                            Some(Ok(tokio_tungstenite::tungstenite::Message::Text(txt))) => {
                                if let Some(ev) = parse_frame(txt.as_str()).map(|ev| source.label(ev)) {
                                    if matches!(ev, RealtimeEvent::Transcript { .. }) {
                                        backoff.reset();
                                    }
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
                crate::diag::log(&format!("realtime connect failed: {e}"));
                let _ = events_out.try_send(RealtimeEvent::Error { message: e.to_string() });
            }
        }
        tokio::time::sleep(backoff.next_delay()).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_transcription_models_are_passed_through() {
        assert_eq!(transcription_model("gpt-transcribe"), "gpt-transcribe");
        assert_eq!(transcription_model(" gpt-live-transcribe "), "gpt-live-transcribe");
        assert_eq!(DEFAULT_TRANSCRIPTION_MODEL, "gpt-4o-mini-transcribe");
        // What earlier versions saved in settings.
        assert_eq!(transcription_model("gpt-4o-mini-realtime"), DEFAULT_TRANSCRIPTION_MODEL);
        assert_eq!(transcription_model(""), DEFAULT_TRANSCRIPTION_MODEL);
    }

    fn transcription(model: &str, language: &str) -> Transcription {
        Transcription { model: model.into(), language: language.into() }
    }

    #[test]
    fn the_session_is_a_transcription_session_without_server_turn_detection() {
        let v = session_update(&transcription("gpt-4o-mini-realtime", ""), Source::Remote);
        let input = &v["session"]["audio"]["input"];
        assert_eq!(v["session"]["type"], "transcription");
        assert_eq!(input["format"]["rate"], 24_000);
        assert_eq!(input["transcription"]["model"], DEFAULT_TRANSCRIPTION_MODEL);
        assert!(input["turn_detection"].is_null());
    }

    #[test]
    fn noise_reduction_suits_each_source() {
        let remote = session_update(&transcription("", ""), Source::Remote);
        let mine = session_update(&transcription("", ""), Source::Mine);
        assert_eq!(remote["session"]["audio"]["input"]["noise_reduction"], serde_json::json!({ "type": "near_field" }));
        assert_eq!(mine["session"]["audio"]["input"]["noise_reduction"], serde_json::json!({ "type": "far_field" }));
    }

    #[test]
    fn the_call_language_is_sent_only_when_known() {
        let config = |model: &str, language: &str| {
            session_update(&transcription(model, language), Source::Remote)["session"]["audio"]["input"]["transcription"]
                .clone()
        };
        // Auto: the model detects the language, as before.
        assert!(config("", "").get("language").is_none());
        assert!(config("", "fr").get("language").is_none());
        assert_eq!(config("", "es")["language"], "es");
        assert_eq!(config("gpt-transcribe", " en ")["language"], "en");
        // `gpt-live-transcribe` takes a list instead, and never both.
        let live = config("gpt-live-transcribe", "en");
        assert_eq!(live["languages"], serde_json::json!(["en"]));
        assert!(live.get("language").is_none());
    }
}
