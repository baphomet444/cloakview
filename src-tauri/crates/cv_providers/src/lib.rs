//! One provider, one key, two jobs: listen (STT) and think (LLM).
//!
//! The rest of CloakView only talks to `dyn Provider`. Which vendor sits
//! underneath is decided by a `ProviderProfile` the user picks once.

pub mod audio;
pub mod openai_compat;
pub mod pipeline;
pub mod profiles;
pub mod redact;
pub mod secrets;
pub mod sse;
pub mod vad;
pub mod wav;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("provider '{0}' does not support speech-to-text; pick another provider")]
    NoSttSupport(String),
    #[error("no API key stored for provider '{0}'")]
    MissingKey(String),
    #[error("provider '{0}' is not a known profile")]
    UnknownProfile(String),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    /// The response body is kept for classification only — the `Display`
    /// never includes it, and `user_message` maps it to a fixed category
    /// with our own text (some vendors echo a key fragment in 401 bodies).
    #[error("the provider request failed (HTTP {status})")]
    Api { status: u16, body: String },
    #[error("unexpected response shape: {0}")]
    Shape(String),
}

impl ProviderError {
    /// A fixed-category, key-free message safe for the UI and the
    /// diagnostics log: the provider's body never crosses this boundary
    /// (see `redact`), and the result is scrubbed as a second defense.
    pub fn user_message(&self) -> String {
        use redact::{classify, scrub_secrets, FailureKind};
        let msg = match self {
            ProviderError::NoSttSupport(id) => format!("provider '{id}' has no speech-to-text model"),
            ProviderError::MissingKey(id) => format!("no API key stored for provider '{id}'"),
            ProviderError::UnknownProfile(id) => format!("unknown provider '{id}'"),
            ProviderError::Http(_) => FailureKind::Network.message(None),
            ProviderError::Api { status, body } => classify(*status, body).message(Some(*status)),
            ProviderError::Shape(m) => format!("unexpected response shape: {m}"),
        };
        scrub_secrets(&msg)
    }
}

/// How a provider transcribes. `Streaming` providers have a realtime
/// endpoint and stream text during the turn; `Batch` providers transcribe
/// one finished segment per request. The app keeps its proven streaming
/// path for `Streaming` and routes `Batch` through the segment pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SttMode {
    Streaming,
    Batch,
}

/// One short audio segment (already VAD-cut, 16 kHz mono PCM16).
pub struct AudioSegment {
    pub pcm16_mono_16k: Vec<i16>,
    /// Which stream it came from, so you get "me vs them" without diarization.
    pub source: AudioSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioSource {
    Microphone,
    SystemAudio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    pub text: String,
    pub source: AudioSource,
}

/// One part of a message. `Image` carries a data URL (the screen-read
/// feature); text is everything else.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Part {
    Text(String),
    /// `data:image/...;base64,...` — the provider receives it as an image.
    Image(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "system" | "user" | "assistant"
    pub parts: Vec<Part>,
}

impl ChatMessage {
    pub fn text(role: &str, content: impl Into<String>) -> Self {
        Self { role: role.into(), parts: vec![Part::Text(content.into())] }
    }
}

/// What the model is asked: `instructions` shape the answer (the app's
/// copilot prompt), `messages` are the conversation or the screen.
#[derive(Debug, Clone, Default)]
pub struct SuggestionRequest {
    pub instructions: String,
    pub messages: Vec<ChatMessage>,
}

/// One item of a streaming suggestion. `Delta` carries text as it is
/// generated; `Done` ends the stream successfully; `Failed` ends it with a
/// non-secret message safe to show the user.
#[derive(Debug, Clone, PartialEq)]
pub enum SuggestionEvent {
    Delta(String),
    Done,
    Failed(String),
}

#[async_trait]
pub trait Provider: Send + Sync {
    fn id(&self) -> &str;
    fn stt_mode(&self) -> SttMode;
    fn supports_stt(&self) -> bool;

    /// Listen: turn one audio segment into text.
    async fn transcribe(&self, seg: &AudioSegment, language: Option<&str>)
        -> Result<Transcript, ProviderError>;

    /// Think: produce a suggestion from the conversation so far. The default
    /// buffers `suggest_stream`; providers override it when they have nothing
    /// better, and override `suggest_stream` for true streaming.
    async fn suggest(&self, request: &SuggestionRequest) -> Result<String, ProviderError> {
        let mut rx = self.suggest_stream(request).await?;
        let mut text = String::new();
        while let Some(event) = rx.recv().await {
            match event {
                SuggestionEvent::Delta(d) => text.push_str(&d),
                SuggestionEvent::Done => break,
                SuggestionEvent::Failed(m) => {
                    return Err(ProviderError::Shape(format!("stream failed: {m}")))
                }
            }
        }
        Ok(text)
    }

    /// Think, streaming. The default implementation makes one non-streaming
    /// `suggest` call and emits it as a single `Delta` followed by `Done` —
    /// correct for every provider, optimal for none.
    async fn suggest_stream(
        &self,
        request: &SuggestionRequest,
    ) -> Result<mpsc::Receiver<SuggestionEvent>, ProviderError> {
        let text = self.suggest(request).await?;
        let (tx, rx) = mpsc::channel(4);
        tokio::spawn(async move {
            let _ = tx.send(SuggestionEvent::Delta(text)).await;
            let _ = tx.send(SuggestionEvent::Done).await;
        });
        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exact scenario the redaction rules exist for: a 401 body that
    /// echoes a masked fragment of the key. Neither the fragment nor the
    /// body may reach anything `Display` or `user_message` produces.
    #[test]
    fn a_401_body_echoing_a_key_fragment_never_reaches_any_message() {
        let fragment = "sk-AbCdEf1234GhIjKl5678mnop";
        let body = format!(
            r#"{{"error":{{"message":"Incorrect API key provided: {fragment}. You can find your API key at https://platform.openai.com/account/api-keys.","type":"invalid_request_error","param":null,"code":"invalid_api_key"}}}}"#
        );
        let err = ProviderError::Api { status: 401, body };

        let display = err.to_string();
        let message = err.user_message();
        for surface in [&display, &message] {
            assert!(!surface.contains(fragment), "raw body/fragment leaked: {surface}");
            assert!(!surface.contains("platform.openai.com"), "body leaked: {surface}");
            assert!(!surface.contains("sk-AbCdEf"), "fragment leaked: {surface}");
        }
        assert!(message.contains("401"), "the status helps the user; keep it: {message}");
        assert!(message.contains("key"), "category should say it is a key problem: {message}");
    }

    #[test]
    fn user_message_maps_each_category() {
        let api = |status: u16, body: &str| ProviderError::Api { status, body: body.to_string() };
        assert!(api(401, "").user_message().contains("key"));
        assert!(api(429, "").user_message().contains("rate limited"));
        assert!(api(404, "").user_message().contains("model"));
        assert!(api(500, "internal").user_message().contains("HTTP 500"));
        assert!(matches!(
            ProviderError::NoSttSupport("groq".into()).user_message().as_str(),
            m if m.contains("groq") && m.contains("speech-to-text")
        ));
    }
}
