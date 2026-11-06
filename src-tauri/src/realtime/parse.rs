use crate::realtime::RealtimeEvent;

/// Parses one realtime WS frame into an event, or `None` for frame types
/// this app doesn't act on. Pure — no I/O, unit-tested directly.
pub fn parse_frame(json: &str) -> Option<RealtimeEvent> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let t = v.get("type")?.as_str()?;
    match t {
        "session.started" => Some(RealtimeEvent::SessionStarted),
        "conversation.item.input_audio_transcription.completed" => {
            let item = v.get("item")?;
            let text = item
                .get("content")?
                .as_array()?
                .iter()
                .filter_map(|c| c.get("text")?.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            if text.trim().is_empty() {
                None
            } else {
                Some(RealtimeEvent::Transcript {
                    speaker: "remote".into(), // speaker attribution is heuristic in v1
                    text: text.trim().into(),
                    is_user_turn: false,
                })
            }
        }
        "error" => Some(RealtimeEvent::Error {
            message: v.get("error")?.get("message")?.as_str()?.to_string(),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_completed_transcript() {
        let frame = r#"{"type":"conversation.item.input_audio_transcription.completed",
            "item":{"content":[{"type":"input_text","text":"Hola, ¿me escuchan?"}]}}"#;
        match parse_frame(frame) {
            Some(RealtimeEvent::Transcript { text, .. }) => assert_eq!(text, "Hola, ¿me escuchan?"),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn malformed_frames_yield_none_or_error_event() {
        assert!(parse_frame("not json").is_none());
        assert!(parse_frame("{}").is_none());
        let err = r#"{"type":"error","error":{"message":"rate_limit"}}"#;
        assert!(matches!(parse_frame(err), Some(RealtimeEvent::Error { .. })));
    }
}
