use serde::Serialize;
use std::time::Duration;

pub mod client;
pub mod parse;

/// Events surfaced from the realtime WS session, emitted to the frontend
/// under the `"realtime"` event name.
///
/// `Error` is a struct variant (`Error { message: String }`), not a tuple
/// variant (`Error(String)`), even though Task 9's "Interfaces" prose
/// summary in the plan wrote the tuple form. With `#[serde(tag = "kind")]`,
/// a tuple variant serializes as `{"kind":"Error","0":"..."}` — the `"0"`
/// key breaks the frontend contract (`models.ts`, Task 12) expecting a
/// named `message` field. The plan's own code sketch already used the
/// struct form; this keeps that, not the prose.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RealtimeEvent {
    Transcript { speaker: String, text: String, is_user_turn: bool },
    TurnBoundary,
    Error { message: String },
    SessionStarted,
    Closed,
}

/// Reconnect backoff: 1s → 2s → 4s → 8s → capped at 15s, `reset()` restarts
/// the sequence at 1s.
#[derive(Debug, Default)]
pub struct Backoff {
    attempt: u32,
}

impl Backoff {
    /// Explicit table, not a bit-shift formula: the earlier bit-shift draft
    /// (`1 << attempt.min(3)`, pre-incrementing `attempt` before computing
    /// the delay) never produces 1s and plateaus at 8s instead of the
    /// spec'd 15s cap — see `task-9-report.md` for the red run against this
    /// test. A match table makes each step's value an explicit, checkable
    /// fact instead of an off-by-one-prone formula.
    pub fn next_delay(&mut self) -> Duration {
        let secs = match self.attempt {
            0 => 1,
            1 => 2,
            2 => 4,
            3 => 8,
            _ => 15,
        };
        self.attempt += 1;
        Duration::from_secs(secs)
    }

    pub fn reset(&mut self) {
        self.attempt = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Written from the Task 9 spec (1s → 2s → 4s → 8s → cap 15s, reset
    /// restarts at 1s) rather than from the plan's example implementation.
    #[test]
    fn backoff_follows_spec_sequence_and_resets() {
        let mut b = Backoff::default();
        let seq: Vec<u64> = (0..6).map(|_| b.next_delay().as_secs()).collect();
        assert_eq!(seq, vec![1, 2, 4, 8, 15, 15]);
        b.reset();
        assert_eq!(b.next_delay().as_secs(), 1);
    }
}
