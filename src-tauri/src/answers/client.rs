use crate::answers::{parse::parse_sse_line, parse::LineBuffer, AnswersEvent};
use crate::transcript::Turn;
use futures_util::StreamExt;
use tokio::sync::mpsc::error::TrySendError;

const URL: &str = "https://api.openai.com/v1/responses";
const CHARS_PER_TOKEN: usize = 4;
const CONTEXT_TOKEN_BUDGET: usize = 2_000;

/// Builds the Responses API `input` array from recent context, budgeted
/// by (approximate) tokens rather than turn count: walk `context` from
/// most recent backward, keep accepting turns while under
/// `CONTEXT_TOKEN_BUDGET * CHARS_PER_TOKEN` chars, then restore
/// chronological order before appending the live `question`.
///
/// `chars/4` is an approximation, not a real tokenizer — fine for a
/// soft context budget, not for exact token accounting.
///
/// Budget is counted in `chars()`, not bytes (`str::len()`): a multi-byte
/// UTF-8 character such as `'á'` is 2 bytes but 1 char, so measuring by
/// byte length burns the budget roughly twice as fast as intended for
/// accented text. When the next (older) turn would overrun the remaining
/// budget, it is truncated to exactly what's left and kept — never
/// dropped outright, even when it's the single most recent turn and the
/// first one considered.
fn build_input(context: &[Turn], question: Option<&str>) -> Vec<serde_json::Value> {
    let mut budget_chars = CONTEXT_TOKEN_BUDGET * CHARS_PER_TOKEN;
    let mut selected: Vec<(bool, String)> = Vec::new();
    for turn in context.iter().rev() {
        if budget_chars == 0 {
            break;
        }
        let char_count = turn.text.chars().count();
        if char_count <= budget_chars {
            budget_chars -= char_count;
            selected.push((turn.is_user_turn, turn.text.clone()));
        } else {
            let truncated: String = turn.text.chars().take(budget_chars).collect();
            budget_chars = 0;
            selected.push((turn.is_user_turn, truncated));
        }
    }
    selected.reverse();

    let mut input: Vec<serde_json::Value> = selected
        .into_iter()
        .map(|(is_user_turn, text)| {
            serde_json::json!({
                "role": if is_user_turn { "user" } else { "assistant" },
                "content": text,
            })
        })
        .collect();

    if let Some(q) = question {
        input.push(serde_json::json!({ "role": "user", "content": q }));
    }
    input
}

/// Spawns the Responses API streaming request and returns a bounded
/// receiver of parsed events plus the task's `JoinHandle`.
///
/// Uses `mpsc::channel(64)` (bounded, not unbounded) with `try_send`:
/// - `Full`: drop this delta; the next successful delta still carries the
///   UI forward, and blocking here would stall the HTTP read loop on a
///   slow/stuck frontend.
/// - `Closed`: stop the task immediately instead of draining a live HTTP
///   stream to completion nobody is listening to anymore.
pub fn stream_answer(
    api_key: String,
    model: String,
    context: Vec<Turn>,
    question: Option<String>,
) -> (tokio::sync::mpsc::Receiver<AnswersEvent>, tokio::task::JoinHandle<()>) {
    let (tx, rx) = tokio::sync::mpsc::channel(64);

    let handle = tokio::spawn(async move {
        let input = build_input(&context, question.as_deref());
        let body = serde_json::json!({
            "model": model,
            "input": input,
            "stream": true,
        });

        let resp = reqwest::Client::new()
            .post(URL)
            .header("Authorization", format!("Bearer {api_key}"))
            .json(&body)
            .send()
            .await;

        let resp = match resp {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.try_send(AnswersEvent::Error { message: e.to_string() });
                return;
            }
        };

        let mut stream = resp.bytes_stream();
        let mut buf = LineBuffer::default();

        while let Some(chunk) = stream.next().await {
            let bytes = match chunk {
                Ok(b) => b,
                Err(e) => {
                    let _ = tx.try_send(AnswersEvent::Error { message: e.to_string() });
                    return;
                }
            };
            let text = String::from_utf8_lossy(&bytes);

            for line in buf.push(&text) {
                let Some(event) = parse_sse_line(&line) else { continue };
                let done = matches!(event, AnswersEvent::Done | AnswersEvent::Error { .. });

                match tx.try_send(event) {
                    Ok(()) => {}
                    Err(TrySendError::Full(_)) => {
                        // Drop this delta; keep reading. A later delta (or
                        // Done/Error) still reaches the receiver.
                    }
                    Err(TrySendError::Closed(_)) => return,
                }

                if done {
                    return;
                }
            }
        }
    });

    (rx, handle)
}

#[cfg(test)]
mod build_input_tests {
    use super::build_input;
    use crate::transcript::Turn;

    fn turn(is_user_turn: bool, text: String, ts: i64) -> Turn {
        Turn { speaker: "s".into(), text, is_user_turn, ts }
    }

    #[test]
    fn keeps_most_recent_turns_within_budget() {
        let context: Vec<Turn> = (0..20)
            .map(|i| turn(i % 2 == 0, "x".repeat(500), i as i64))
            .collect();
        let input = build_input(&context, None);

        assert!(!input.is_empty(), "budget starved out every turn");
        assert!(input.len() < 20, "all 20 turns fit: budget math is wrong");

        let newest = context.last().unwrap();
        assert_eq!(
            input.last().unwrap()["content"],
            serde_json::json!(newest.text),
            "last entry should be the most recent turn"
        );
    }

    #[test]
    fn oversized_newest_turn_is_truncated_not_dropped() {
        let context = vec![turn(true, "x".repeat(200_000), 0)];
        let input = build_input(&context, None);

        assert_eq!(
            input.len(),
            1,
            "an oversized newest turn must be truncated, never dropped entirely"
        );
        let content = input[0]["content"].as_str().unwrap();
        assert!(
            content.chars().count() < 200_000,
            "expected truncation, got the full {}-char turn",
            content.chars().count()
        );
    }

    #[test]
    fn counts_chars_not_bytes() {
        // CONTEXT_TOKEN_BUDGET * CHARS_PER_TOKEN chars; 'á' is 2 bytes in
        // UTF-8, so measuring by .len() (bytes) would see 1.5x the budget
        // and drop this turn entirely instead of keeping it whole.
        let budget_chars = super::CONTEXT_TOKEN_BUDGET * super::CHARS_PER_TOKEN;
        let n_chars = budget_chars * 3 / 4;
        let text: String = "á".repeat(n_chars);
        let context = vec![turn(true, text.clone(), 0)];

        let input = build_input(&context, None);

        assert_eq!(input.len(), 1, "turn should fit whole, not be dropped");
        assert_eq!(
            input[0]["content"].as_str().unwrap().chars().count(),
            n_chars,
            "turn should not be truncated either -- it fits under the char budget"
        );
    }

    #[test]
    fn question_is_appended_as_user_turn() {
        let input = build_input(&[], Some("q"));
        assert_eq!(
            input,
            vec![serde_json::json!({ "role": "user", "content": "q" })]
        );
    }
}
