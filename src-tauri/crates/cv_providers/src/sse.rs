//! Server-Sent Events parsing for streaming suggestions, shared by every
//! OpenAI-compatible flavor. Two dialects exist and both live here:
//!
//! - **Responses API** (`response.output_text.delta` / `response.completed` /
//!   `response.failed` / bare `error`): no `[DONE]` sentinel.
//! - **Chat Completions** (`choices[0].delta.content`, `[DONE]` sentinel).
//!
//! Moved from the app's `answers::parse` (Responses dialect, same tests) and
//! extended with the Chat dialect for Groq/Together-style providers.

use crate::SuggestionEvent;

/// Buffers raw stream bytes and yields only newline-terminated lines,
/// retaining any trailing partial line for the next `push`.
///
/// Required because `reqwest::Response::bytes_stream()` chunk boundaries
/// are a transport artifact and do not align with SSE line boundaries — a
/// single `data: {...}` line can arrive split across two chunks.
#[derive(Debug, Default)]
pub struct LineBuffer {
    buf: String,
}

impl LineBuffer {
    /// Feed a chunk of (already UTF-8–decoded) stream text. Returns
    /// complete lines (newline stripped) found since the buffer started;
    /// an incomplete trailing line stays buffered.
    pub fn push(&mut self, chunk: &str) -> Vec<String> {
        self.buf.push_str(chunk);
        let mut lines = Vec::new();
        while let Some(idx) = self.buf.find('\n') {
            let line: String = self.buf.drain(..=idx).collect();
            lines.push(line.trim_end_matches(['\n', '\r']).to_string());
        }
        lines
    }
}

/// Parses one complete SSE line from the Responses API stream.
///
/// `response.output_text.delta` -> `Delta`, `response.completed` -> `Done`,
/// `response.failed` -> `Failed` (message nested under `response.error`),
/// bare `error` -> `Failed` (message at top level). No `"data: [DONE]"`
/// sentinel in this API.
pub fn parse_responses_line(line: &str) -> Option<SuggestionEvent> {
    let body = line.strip_prefix("data:")?.trim();
    if body.is_empty() {
        return None;
    }
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    let t = v.get("type")?.as_str()?;
    match t {
        "response.output_text.delta" => {
            let text = v.get("delta")?.as_str()?.to_string();
            Some(SuggestionEvent::Delta(text))
        }
        "response.completed" => Some(SuggestionEvent::Done),
        "response.failed" => {
            let message = v
                .get("response")?
                .get("error")?
                .get("message")?
                .as_str()?
                .to_string();
            Some(SuggestionEvent::Failed(message))
        }
        "error" => {
            let message = v.get("message")?.as_str()?.to_string();
            Some(SuggestionEvent::Failed(message))
        }
        _ => None,
    }
}

/// Parses one complete SSE line from a Chat Completions stream.
///
/// `data: [DONE]` -> `Done`; `choices[0].delta.content` -> `Delta` (skipped
/// when empty: role/presence deltas carry no content); an `error` object ->
/// `Failed` with the provider's message.
pub fn parse_chat_line(line: &str) -> Option<SuggestionEvent> {
    let body = line.strip_prefix("data:")?.trim();
    if body.is_empty() {
        return None;
    }
    if body == "[DONE]" {
        return Some(SuggestionEvent::Done);
    }
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    if let Some(message) = v.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()) {
        return Some(SuggestionEvent::Failed(message.to_string()));
    }
    let content = v
        .get("choices")?
        .get(0)?
        .get("delta")?
        .get("content")?
        .as_str()?;
    if content.is_empty() {
        None
    } else {
        Some(SuggestionEvent::Delta(content.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_output_text_delta() {
        let line = r#"data: {"type":"response.output_text.delta","delta":"Hola"}"#;
        assert_eq!(parse_responses_line(line), Some(SuggestionEvent::Delta("Hola".into())));
    }

    #[test]
    fn response_completed_is_done() {
        let line = r#"data: {"type":"response.completed"}"#;
        assert_eq!(parse_responses_line(line), Some(SuggestionEvent::Done));
    }

    #[test]
    fn response_failed_has_nested_error_message() {
        let line = r#"data: {"type":"response.failed","response":{"error":{"message":"rate_limit"}}}"#;
        assert_eq!(parse_responses_line(line), Some(SuggestionEvent::Failed("rate_limit".into())));
    }

    #[test]
    fn bare_error_type_has_top_level_message() {
        let line = r#"data: {"type":"error","message":"server_error"}"#;
        assert_eq!(parse_responses_line(line), Some(SuggestionEvent::Failed("server_error".into())));
    }

    #[test]
    fn split_line_parses_once_completed() {
        let mut buf = LineBuffer::default();
        let first = buf.push(r#"data: {"type":"response.output_text.delta","del"#);
        assert!(first.is_empty(), "no complete line yet, got {first:?}");

        let second = buf.push("ta\":\"Hola\"}\n");
        assert_eq!(second.len(), 1);
        assert_eq!(parse_responses_line(&second[0]), Some(SuggestionEvent::Delta("Hola".into())));
    }

    #[test]
    fn keepalive_and_empty_line_yield_none() {
        assert!(parse_responses_line(": keepalive").is_none());
        assert!(parse_responses_line("").is_none());
        assert!(parse_chat_line(": keepalive").is_none());
    }

    #[test]
    fn broken_json_yields_none_without_panic() {
        assert!(parse_responses_line("data: {broken").is_none());
        assert!(parse_chat_line("data: {broken").is_none());
    }

    #[test]
    fn chat_done_sentinel_is_done() {
        assert_eq!(parse_chat_line("data: [DONE]"), Some(SuggestionEvent::Done));
    }

    #[test]
    fn chat_delta_content_streams() {
        let line = r#"data: {"choices":[{"index":0,"delta":{"content":"Hello"}}]}"#;
        assert_eq!(parse_chat_line(line), Some(SuggestionEvent::Delta("Hello".into())));
    }

    #[test]
    fn chat_role_and_empty_deltas_are_skipped() {
        let role = r#"data: {"choices":[{"index":0,"delta":{"role":"assistant"}}]}"#;
        assert_eq!(parse_chat_line(role), None);
        let empty = r#"data: {"choices":[{"index":0,"delta":{"content":""}}]}"#;
        assert_eq!(parse_chat_line(empty), None);
    }

    #[test]
    fn chat_error_object_fails_with_message() {
        let line = r#"data: {"error":{"message":"model_not_found","type":"invalid_request_error"}}"#;
        assert_eq!(parse_chat_line(line), Some(SuggestionEvent::Failed("model_not_found".into())));
    }
}
