//! The Rust → frontend event names, exactly as design.md §5 ("Eventos Rust →
//! frontend") spells them. Each is its own Tauri event with a bare payload, so
//! the frontend registers one listener per event and never switches on a
//! `kind` field inside it.
//!
//! Tauri's `emit` is only the transport. Internally the realtime and answers
//! streams travel over `mpsc` channels (`RealtimeEvent`, `AnswersEvent`);
//! `listen::pump` and `commands::request_answer` are the only places that map
//! those onto these names. The old catch-all `"realtime"` / `"answers"` events
//! no longer exist — nothing emits them.
//!
//! Every `emit` call must use one of these constants rather than a string
//! literal, so a typo is a compile error instead of a silently dead listener.

pub const TRANSCRIPT_DELTA_EVENT: &str = "transcript-delta";
pub const TURN_DETECTED_EVENT: &str = "turn-detected";
pub const ANSWER_DELTA_EVENT: &str = "answer-delta";
pub const ANSWER_DONE_EVENT: &str = "answer-done";
/// Not in design.md §5: an answer the user did not type is starting (today: one read
/// from the screen), so Ask can show it. Payload: what it answers (`"screen"`).
pub const ANSWER_STARTED_EVENT: &str = "answer-started";
pub const LISTEN_STATE_EVENT: &str = "listen-state";
pub const ERROR_EVENT: &str = "error";
/// Not in design.md §5: the push-to-ask global shortcut asks the hub to open the Ask
/// window, so it goes through the same path as the hub's Ask button. No payload.
pub const PUSH_TO_ASK_EVENT: &str = "push-to-ask";

#[cfg(test)]
mod tests {
    use super::*;

    /// design.md §5 names the first six; `push-to-ask` (Task 15) is the only addition. An independent literal
    /// copy, so editing a constant above (or the catch-all channels coming
    /// back) trips this instead of silently breaking a frontend listener.
    #[test]
    fn event_names_are_exactly_the_spec_events() {
        let ours = [
            TRANSCRIPT_DELTA_EVENT,
            TURN_DETECTED_EVENT,
            ANSWER_DELTA_EVENT,
            ANSWER_DONE_EVENT,
            ANSWER_STARTED_EVENT,
            LISTEN_STATE_EVENT,
            ERROR_EVENT,
            PUSH_TO_ASK_EVENT,
        ];
        let spec = [
            "transcript-delta",
            "turn-detected",
            "answer-delta",
            "answer-done",
            "answer-started",
            "listen-state",
            "error",
            "push-to-ask",
        ];
        assert_eq!(ours, spec);

        let distinct: std::collections::BTreeSet<_> = ours.iter().collect();
        assert_eq!(distinct.len(), ours.len(), "two events share a name");
        assert!(!ours.contains(&"realtime") && !ours.contains(&"answers"));
    }
}
