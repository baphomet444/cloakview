use serde::Serialize;

pub mod client;
pub mod parse;

/// Events surfaced from the Responses API stream. They travel from
/// `client::stream_answer` to `commands::request_answer` over an `mpsc`
/// channel — this enum is internal transport only. `request_answer` maps each
/// variant onto its design.md §5 frontend event (`answer-delta`,
/// `answer-done`, `error`); see `crate::events`. Nothing is emitted under an
/// `"answers"` event name.
///
/// All three variants are struct variants, including `Delta { text }` —
/// not a newtype `Delta(String)`. `#[serde(tag = "kind")]` is internal
/// tagging, which does not support newtype variants at all (serde errors
/// at runtime trying to serialize one), so this isn't a style choice.
/// See `realtime::RealtimeEvent::Error` (Task 9) for the same rule applied
/// to a different enum.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AnswersEvent {
    Delta { text: String },
    Done,
    Error { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mandatory per task-10-brief.md: compares the *actual* serialized
    /// JSON, not just a pattern-matched Rust variant construction. This
    /// caught something real on the first run: `rename_all = "camelCase"`
    /// under internal tagging also lowercases the variant name used as
    /// the tag value, so `Delta` serializes as `"kind":"delta"`, not
    /// `"Delta"` — a pattern-match-only test (`matches!(ev,
    /// AnswersEvent::Delta { .. })`) can't see that; only comparing the
    /// real JSON does. It would also catch the newtype-variant mistake the
    /// brief warns about (`Delta(String)` serializing as
    /// `{"kind":"delta","0":"Hola"}` instead of a named `text` field).
    #[test]
    fn delta_serializes_with_named_text_field() {
        let ev = AnswersEvent::Delta { text: "Hola".into() };
        let json = serde_json::to_value(&ev).unwrap();
        assert_eq!(json, serde_json::json!({"kind": "delta", "text": "Hola"}));
    }
}
