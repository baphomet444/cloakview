use crate::answers::AnswersEvent;

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
/// `response.failed` -> `Error` (message nested under `response.error`),
/// bare `error` -> `Error` (message at top level). No `"data: [DONE]"`
/// sentinel in this API — unlike Chat Completions, which this is NOT.
pub fn parse_sse_line(line: &str) -> Option<AnswersEvent> {
    let body = line.strip_prefix("data:")?.trim();
    if body.is_empty() {
        return None;
    }
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    let t = v.get("type")?.as_str()?;
    match t {
        "response.output_text.delta" => {
            let text = v.get("delta")?.as_str()?.to_string();
            Some(AnswersEvent::Delta { text })
        }
        "response.completed" => Some(AnswersEvent::Done),
        "response.failed" => {
            let message = v
                .get("response")?
                .get("error")?
                .get("message")?
                .as_str()?
                .to_string();
            Some(AnswersEvent::Error { message })
        }
        "error" => {
            let message = v.get("message")?.as_str()?.to_string();
            Some(AnswersEvent::Error { message })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_output_text_delta() {
        let line = r#"data: {"type":"response.output_text.delta","delta":"Hola"}"#;
        assert_eq!(
            parse_sse_line(line),
            Some(AnswersEvent::Delta { text: "Hola".into() })
        );
    }

    #[test]
    fn response_completed_is_done() {
        let line = r#"data: {"type":"response.completed"}"#;
        assert_eq!(parse_sse_line(line), Some(AnswersEvent::Done));
    }

    #[test]
    fn response_failed_has_nested_error_message() {
        let line = r#"data: {"type":"response.failed","response":{"error":{"message":"rate_limit"}}}"#;
        assert_eq!(
            parse_sse_line(line),
            Some(AnswersEvent::Error { message: "rate_limit".into() })
        );
    }

    #[test]
    fn bare_error_type_has_top_level_message() {
        let line = r#"data: {"type":"error","message":"server_error"}"#;
        assert_eq!(
            parse_sse_line(line),
            Some(AnswersEvent::Error { message: "server_error".into() })
        );
    }

    #[test]
    fn split_line_parses_once_completed() {
        let mut buf = LineBuffer::default();
        let first = buf.push(r#"data: {"type":"response.output_text.delta","del"#);
        assert!(first.is_empty(), "no complete line yet, got {first:?}");

        let second = buf.push("ta\":\"Hola\"}\n");
        assert_eq!(second.len(), 1);
        assert_eq!(
            parse_sse_line(&second[0]),
            Some(AnswersEvent::Delta { text: "Hola".into() })
        );
    }

    #[test]
    fn keepalive_and_empty_line_yield_none() {
        assert!(parse_sse_line(": keepalive").is_none());
        assert!(parse_sse_line("").is_none());
    }

    #[test]
    fn broken_json_yields_none_without_panic() {
        assert!(parse_sse_line("data: {broken").is_none());
    }
}
