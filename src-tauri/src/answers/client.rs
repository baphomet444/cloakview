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
fn build_input(context: &[Turn], question: Option<&str>) -> Vec<serde_json::Value> {
    let mut budget_chars = CONTEXT_TOKEN_BUDGET * CHARS_PER_TOKEN;
    let mut selected: Vec<&Turn> = Vec::new();
    for turn in context.iter().rev() {
        let cost = turn.text.len();
        if cost > budget_chars {
            break;
        }
        selected.push(turn);
        budget_chars -= cost;
    }
    selected.reverse();

    let mut input: Vec<serde_json::Value> = selected
        .into_iter()
        .map(|t| {
            serde_json::json!({
                "role": if t.is_user_turn { "user" } else { "assistant" },
                "content": t.text,
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
