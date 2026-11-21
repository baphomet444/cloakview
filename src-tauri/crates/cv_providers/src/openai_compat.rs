use crate::{
    profiles::{ApiFlavor, ProviderProfile},
    sse::{parse_chat_line, parse_responses_line, LineBuffer},
    wav::pcm16_to_wav,
    AudioSegment, ChatMessage, Part, Provider, ProviderError, SttMode, SuggestionEvent,
    SuggestionRequest, Transcript,
};
use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::multipart::{Form, Part as MultipartPart};
use serde_json::{json, Value};
use tokio::sync::mpsc;

/// Works for any vendor exposing /audio/transcriptions and either the
/// Responses API or the Chat Completions API (OpenAI, Groq, Together,
/// Fireworks, ...). The key is passed in by the caller after being read
/// from the Keychain in Rust.
pub struct OpenAiCompat {
    profile: ProviderProfile,
    api_key: String,
    http: reqwest::Client,
}

impl OpenAiCompat {
    pub fn new(profile: ProviderProfile, api_key: String) -> Self {
        Self { profile, api_key, http: reqwest::Client::new() }
    }

    async fn check(resp: reqwest::Response) -> Result<Value, ProviderError> {
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ProviderError::Api { status: status.as_u16(), body });
        }
        Ok(resp.json::<Value>().await?)
    }

    // --- request bodies, one per flavor ---

    /// Responses API input item, mirroring what the app sent before this
    /// crate existed: plain string content for text-only turns, part arrays
    /// only when an image is present.
    fn responses_input(messages: &[ChatMessage]) -> Value {
        let items: Vec<Value> = messages
            .iter()
            .map(|m| {
                let all_text = m.parts.iter().all(|p| matches!(p, Part::Text(_)));
                if all_text {
                    let text: String = m
                        .parts
                        .iter()
                        .filter_map(|p| match p {
                            Part::Text(t) => Some(t.as_str()),
                            Part::Image(_) => None,
                        })
                        .collect();
                    json!({ "role": m.role, "content": text })
                } else {
                    let content: Vec<Value> = m
                        .parts
                        .iter()
                        .map(|p| match p {
                            Part::Text(t) => json!({ "type": "input_text", "text": t }),
                            Part::Image(url) => json!({ "type": "input_image", "image_url": url }),
                        })
                        .collect();
                    json!({ "role": m.role, "content": content })
                }
            })
            .collect();
        json!(items)
    }

    /// Chat Completions message: string content for text-only turns,
    /// typed parts otherwise. `instructions` become a leading system message
    /// (Chat has no instructions field).
    fn chat_messages(request: &SuggestionRequest) -> Value {
        let mut messages: Vec<Value> = Vec::new();
        if !request.instructions.is_empty() {
            messages.push(json!({ "role": "system", "content": request.instructions }));
        }
        for m in &request.messages {
            let all_text = m.parts.iter().all(|p| matches!(p, Part::Text(_)));
            if all_text {
                let text: String = m
                    .parts
                    .iter()
                    .filter_map(|p| match p {
                        Part::Text(t) => Some(t.as_str()),
                        Part::Image(_) => None,
                    })
                    .collect();
                messages.push(json!({ "role": m.role, "content": text }));
            } else {
                let content: Vec<Value> = m
                    .parts
                    .iter()
                    .map(|p| match p {
                        Part::Text(t) => json!({ "type": "text", "text": t }),
                        Part::Image(url) => json!({ "type": "image_url", "image_url": { "url": url } }),
                    })
                    .collect();
                messages.push(json!({ "role": m.role, "content": content }));
            }
        }
        json!(messages)
    }

    /// The URL and JSON body of one streaming suggestion request.
    fn request_body(&self, request: &SuggestionRequest) -> (String, Value) {
        match self.profile.flavor {
            ApiFlavor::Responses => {
                let mut body = json!({
                    "model": self.profile.llm_model,
                    "instructions": request.instructions,
                    "input": Self::responses_input(&request.messages),
                    "stream": true,
                });
                if let Some(effort) = self.profile.reasoning_effort {
                    body["reasoning"] = json!({ "effort": effort });
                }
                (format!("{}/responses", self.profile.base_url), body)
            }
            ApiFlavor::Chat => (
                format!("{}/chat/completions", self.profile.base_url),
                json!({
                    "model": self.profile.llm_model,
                    "messages": Self::chat_messages(request),
                    "stream": true,
                }),
            ),
        }
    }

    /// One streaming suggestion request, whichever dialect this profile
    /// speaks. The returned receiver is fed parsed `SuggestionEvent`s from a
    /// spawned task; the task stops on `Done`/`Failed` or when the receiver
    /// is dropped.
    async fn stream_events(&self, request: &SuggestionRequest) -> Result<mpsc::Receiver<SuggestionEvent>, ProviderError> {
        let (url, body) = self.request_body(request);

        let resp = self
            .http
            .post(url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body_text = resp.text().await.unwrap_or_default();
            return Err(ProviderError::Api { status: status.as_u16(), body: body_text });
        }

        let parse_line: fn(&str) -> Option<SuggestionEvent> = match self.profile.flavor {
            ApiFlavor::Responses => parse_responses_line,
            ApiFlavor::Chat => parse_chat_line,
        };

        let (tx, rx) = mpsc::channel(64);
        let mut stream = resp.bytes_stream();
        tokio::spawn(async move {
            let mut buf = LineBuffer::default();
            while let Some(chunk) = stream.next().await {
                let bytes = match chunk {
                    Ok(b) => b,
                    Err(e) => {
                        // reqwest stream errors carry no key, but scrub anyway:
                        // this string goes to the UI and the diagnostics log.
                        let _ = tx.try_send(SuggestionEvent::Failed(crate::redact::scrub_secrets(&e.to_string())));
                        return;
                    }
                };
                let text = String::from_utf8_lossy(&bytes);
                for line in buf.push(&text) {
                    let Some(event) = parse_line(&line) else { continue };
                    let stop = matches!(event, SuggestionEvent::Done | SuggestionEvent::Failed(_));
                    // Bounded with try_send: never stall the HTTP read loop on
                    // a slow consumer (drop deltas), but exit promptly when
                    // nobody listens anymore.
                    if tx.try_send(event).is_err() {
                        return;
                    }
                    if stop {
                        return;
                    }
                }
            }
            // The stream ended without a terminal event: report it instead of
            // letting the UI wait for a Done that never comes.
            let _ = tx.try_send(SuggestionEvent::Failed("the stream ended unexpectedly".into()));
        });
        Ok(rx)
    }
}

#[async_trait]
impl Provider for OpenAiCompat {
    fn id(&self) -> &str { &self.profile.id }

    fn stt_mode(&self) -> SttMode { self.profile.stt_mode }

    fn supports_stt(&self) -> bool { self.profile.stt_model.is_some() }

    async fn transcribe(&self, seg: &AudioSegment, language: Option<&str>)
        -> Result<Transcript, ProviderError>
    {
        let model = self.profile.stt_model.clone()
            .ok_or_else(|| ProviderError::NoSttSupport(self.profile.id.clone()))?;

        let wav = pcm16_to_wav(&seg.pcm16_mono_16k, 16_000);
        let part = MultipartPart::bytes(wav).file_name("seg.wav").mime_str("audio/wav")?;
        let mut form = Form::new().part("file", part).text("model", model);
        if let Some(lang) = language {
            form = form.text("language", lang.to_string());
        }

        let resp = self.http
            .post(format!("{}/audio/transcriptions", self.profile.base_url))
            .bearer_auth(&self.api_key)
            .multipart(form)
            .send().await?;
        let v = Self::check(resp).await?;
        let text = v["text"].as_str()
            .ok_or_else(|| ProviderError::Shape("missing 'text'".into()))?
            .trim().to_string();
        Ok(Transcript { text, source: seg.source })
    }

    async fn suggest_stream(
        &self,
        request: &SuggestionRequest,
    ) -> Result<mpsc::Receiver<SuggestionEvent>, ProviderError> {
        self.stream_events(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{profiles::builtin_profiles, AudioSource};

    #[tokio::test]
    async fn llm_only_profile_refuses_stt_cleanly() {
        let mut p = builtin_profiles().into_iter().find(|p| p.id == "groq").unwrap();
        p.stt_model = None;
        let prov = OpenAiCompat::new(p, "k".into());
        assert!(!prov.supports_stt());
        let seg = AudioSegment { pcm16_mono_16k: vec![0; 160], source: AudioSource::Microphone };
        let err = prov.transcribe(&seg, None).await.unwrap_err();
        assert!(matches!(err, ProviderError::NoSttSupport(_)));
    }

    #[test]
    fn openai_asks_a_reasoning_model_with_low_effort() {
        let p = builtin_profiles().into_iter().find(|p| p.id == "openai").unwrap();
        assert_eq!(p.llm_model, "gpt-6-luna");
        let prov = OpenAiCompat::new(p, "k".into());
        let request = SuggestionRequest { instructions: "i".into(), messages: vec![ChatMessage::text("user", "q")] };
        let (url, body) = prov.request_body(&request);
        assert!(url.ends_with("/responses"));
        assert_eq!(body["model"], json!("gpt-6-luna"));
        assert_eq!(body["reasoning"], json!({ "effort": "low" }));
        assert_eq!(body["stream"], json!(true));
    }

    #[test]
    fn other_providers_send_no_reasoning_field() {
        for id in ["groq", "together"] {
            let p = builtin_profiles().into_iter().find(|p| p.id == id).unwrap();
            let prov = OpenAiCompat::new(p, "k".into());
            let request = SuggestionRequest { instructions: "i".into(), messages: vec![ChatMessage::text("user", "q")] };
            let (_, body) = prov.request_body(&request);
            assert!(body.get("reasoning").is_none(), "{id} must not get a reasoning field");
        }
    }

    #[test]
    fn text_only_messages_stay_plain_strings_both_flavors() {
        let messages = vec![
            ChatMessage::text("assistant", "They said hello"),
            ChatMessage::text("user", "How do I reply?"),
        ];
        let request = SuggestionRequest {
            instructions: "You are a copilot.".into(),
            messages,
        };
        let responses = OpenAiCompat::responses_input(&request.messages);
        assert_eq!(responses[0]["content"], json!("They said hello"));
        assert_eq!(responses[1]["content"], json!("How do I reply?"));

        let chat = OpenAiCompat::chat_messages(&request);
        assert_eq!(chat[0], json!({ "role": "system", "content": "You are a copilot." }));
        assert_eq!(chat[1]["content"], json!("They said hello"));
        assert_eq!(chat[2]["content"], json!("How do I reply?"));
    }

    #[test]
    fn image_parts_become_typed_content_both_flavors() {
        let messages = vec![ChatMessage {
            role: "user".into(),
            parts: vec![
                Part::Text("This is my screen.".into()),
                Part::Image("data:image/jpeg;base64,AAAA".into()),
            ],
        }];
        let responses = OpenAiCompat::responses_input(&messages);
        assert_eq!(responses[0]["content"][0]["type"], json!("input_text"));
        assert_eq!(responses[0]["content"][1]["type"], json!("input_image"));
        assert_eq!(
            responses[0]["content"][1]["image_url"],
            json!("data:image/jpeg;base64,AAAA")
        );

        let chat = OpenAiCompat::chat_messages(&SuggestionRequest {
            instructions: String::new(),
            messages,
        });
        assert_eq!(chat[0]["content"][0]["type"], json!("text"));
        assert_eq!(chat[0]["content"][1]["type"], json!("image_url"));
        assert_eq!(chat[0]["content"][1]["image_url"]["url"], json!("data:image/jpeg;base64,AAAA"));
    }

    #[test]
    fn chat_messages_skip_the_system_line_without_instructions() {
        let chat = OpenAiCompat::chat_messages(&SuggestionRequest {
            instructions: String::new(),
            messages: vec![ChatMessage::text("user", "hi")],
        });
        assert_eq!(chat[0]["role"], json!("user"));
    }
}
