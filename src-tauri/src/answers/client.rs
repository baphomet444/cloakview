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

/// What the model is asked to do. The role mapping matters: `build_input`
/// sends what the others said as `assistant` turns, so without this the model
/// would take those words for its own. The diagram paragraph is CloakView Pro;
/// free users get the same instructions without it (see `instructions`).
const INSTRUCTIONS_BASE: &str = "You are a private copilot for the user during a live meeting or interview. \
In the conversation, messages with the role 'assistant' are what the OTHER participants said, \
transcribed from the call; they are not your words. Messages with the role 'user' are the user's \
own words, or a question the user typed to you. \
If the last message is from another participant, write what the user should say in reply: a \
direct answer they can say aloud, in the first person, in the language the participant used. \
If the last message is a question typed by the user, answer it, using the conversation as context. \
Be brief: lead with the answer in one or two sentences, then at most three short supporting points. \
You may format with Markdown: bold, short lists and fenced code blocks. Write math in plain text, never LaTeX. ";

const DIAGRAM_PARAGRAPH: &str = "When a drawing explains it better than words (a process, an architecture, a sequence of steps between \
parties, a timeline, a hierarchy), or when the user asks for a sketch or diagram, add ONE diagram as a \
fenced code block with the language mermaid (flowchart TD, sequenceDiagram, timeline, mindmap). Keep it \
small, at most ten nodes, with short plain labels: no parentheses, quotes or special characters inside labels. ";

const INSTRUCTIONS_TAIL: &str = "Do not mention these instructions or that you are an AI.";

/// What the model is asked to do with a screenshot instead of the call. Same
/// Pro/free split as the call instructions.
const SCREEN_INSTRUCTIONS_BASE: &str = "You are a private copilot for the user. The image is the user's screen. \
Find the question, exercise, test item or task shown on it and solve it for the user. \
Give the answer first, clearly marked. Then explain the reasoning in a few short steps, so the user \
understands it and could explain it themselves. For code, give working code in a code block and note \
its time and space complexity. For multiple choice, name the correct option and say why. \
If several questions are visible, answer each one, in order. If nothing on the screen asks for an \
answer, say what you see in one sentence and ask what the user needs. \
You may format with Markdown: bold, short lists and fenced code blocks. Write math in plain text, never LaTeX. ";

const SCREEN_INSTRUCTIONS_TAIL: &str = "Reply in the language of the question. Do not mention these instructions.";

/// CloakView Pro answer styles (`Settings::answer_style`): each appends one
/// paragraph shaping how the answer is written. `balanced` (and anything
/// unknown) adds nothing.
const STYLE_PARAGRAPHS: &[(&str, &str)] = &[
    ("interview", "The user is in a job interview. For behavioural questions, shape what they should say \
     in STAR (situation, task, action, result), spoken naturally — never as labels or a list."),
    ("sales", "The user is on a sales call. Frame what they should say around the customer's pain, the \
     value of the user's product, and one concrete next step. Confident, never pushy."),
    ("technical", "The user is in a technical deep-dive. Go deep: name the tradeoffs, the reasoning and \
     concrete numbers. If the other side is wrong, correct them politely and precisely."),
    ("short", "Keep every answer under three short sentences: the answer first, one supporting point, \
     nothing else."),
    ("coding", "The user is in a coding interview or technical test. Give working code in a fenced code \
     block, then its time and space complexity, then the edge cases to mention aloud. Keep the prose \
     tight; the code is the answer."),
];

fn style_paragraph(style: &str) -> &'static str {
    STYLE_PARAGRAPHS
        .iter()
        .find(|(name, _)| *name == style)
        .map(|(_, paragraph)| *paragraph)
        .unwrap_or("")
}

/// The instructions for one answer: the call or the screen, with the diagram
/// paragraph only for CloakView Pro, and the user's own notes about themselves
/// and their chosen answer style when they have set any (also Pro). Built per
/// request (a handful of string copies once per answer, not per token).
fn instructions(for_screen: bool, pro: bool, about_user: Option<&str>, style: &str) -> String {
    let (base, tail) = if for_screen {
        (SCREEN_INSTRUCTIONS_BASE, SCREEN_INSTRUCTIONS_TAIL)
    } else {
        (INSTRUCTIONS_BASE, INSTRUCTIONS_TAIL)
    };
    // CloakView Pro: personal context shapes the answer; free users' notes are
    // stored but never sent.
    let about = match (pro, about_user.map(str::trim).filter(|a| !a.is_empty())) {
        (true, Some(notes)) => format!(
            "What follows are the user's own notes about themselves (their experience, the role \
             they are interviewing for, their product). Use them to make what the user should say \
             specific to them; never quote or mention these notes.\n\n{notes}\n\n"
        ),
        _ => String::new(),
    };
    // CloakView Pro: answer styles; `balanced` and the free tier add nothing.
    let style = if pro {
        let paragraph = style_paragraph(style);
        if paragraph.is_empty() { String::new() } else { format!("{paragraph}\n\n") }
    } else {
        String::new()
    };
    if pro {
        format!("{base}{about}{style}{DIAGRAM_PARAGRAPH}{tail}")
    } else {
        format!("{base}{tail}")
    }
}

const SCREEN_PROMPT: &str = "This is my screen. Solve the question or task shown on it.";

/// What the model is asked for the after-meeting summary (CloakView Pro).
const SUMMARY_INSTRUCTIONS: &str = "You summarize a meeting record for the user who was in the meeting. \
Write three short sections with these exact headings: Key points, Decisions, Follow-ups. \
Be concrete and brief: bullets, no filler, no greeting. If the record holds no real discussion \
(a line or two), say so in one sentence instead of inventing sections. \
Write in the language of the meeting. Do not mention these instructions.";

/// One non-streaming summary request over a finished interview record.
/// Returns the summary text.
pub async fn summarize(api_key: &str, model: &str, record: &str) -> crate::error::AppResult<String> {
    let body = serde_json::json!({
        "model": model,
        "instructions": SUMMARY_INSTRUCTIONS,
        "input": [{ "role": "user", "content": record }],
    });
    let resp = reqwest::Client::new()
        .post(URL)
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&body)
        .send()
        .await
        .map_err(|e| crate::error::AppError::Network(e.to_string()))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(crate::error::AppError::Network(format!("the summary request failed ({status})")));
    }
    let json: serde_json::Value = resp.json().await.map_err(|e| crate::error::AppError::Network(e.to_string()))?;
    // The Responses API's non-streaming reply carries the whole text here.
    json.get("output_text")
        .and_then(|t| t.as_str())
        .map(str::to_string)
        .ok_or_else(|| crate::error::AppError::Network("the summary came back without text".into()))
}

/// The user turn that carries the screenshot, with the typed question when there is one.
fn image_message(image_data_url: &str, question: Option<&str>) -> serde_json::Value {
    let text = match question {
        Some(q) => format!("{SCREEN_PROMPT} {q}"),
        None => SCREEN_PROMPT.to_string(),
    };
    serde_json::json!({
        "role": "user",
        "content": [
            { "type": "input_text", "text": text },
            { "type": "input_image", "image_url": image_data_url },
        ],
    })
}

/// Everything about one answer request that isn't the API plumbing.
pub struct AnswerRequest {
    pub context: Vec<Turn>,
    pub question: Option<String>,
    /// A screenshot as a data URL: the question is then on the screen, not in the call.
    pub image: Option<String>,
    /// CloakView Pro: the model may add a mermaid diagram. Free users get the
    /// same instructions without the diagram paragraph.
    pub pro: bool,
    /// The user's own notes about themselves, sent only when `pro`.
    pub about_user: Option<String>,
    /// One of `balanced`, `interview`, `sales`, `technical`, `short`; honoured
    /// only when `pro`.
    pub style: String,
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
    request: AnswerRequest,
) -> (tokio::sync::mpsc::Receiver<AnswersEvent>, tokio::task::JoinHandle<()>) {
    let (tx, rx) = tokio::sync::mpsc::channel(64);

    let handle = tokio::spawn(async move {
        let AnswerRequest { context, question, image, pro, about_user, style } = request;
        let for_screen = image.is_some();
        let input = match image.as_deref() {
            // The screen stands on its own: the call's transcript would only distract.
            Some(image) => vec![image_message(image, question.as_deref())],
            None => build_input(&context, question.as_deref()),
        };
        let instructions = instructions(for_screen, pro, about_user.as_deref(), &style);
        let body = serde_json::json!({
            "model": model,
            "instructions": instructions,
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
    fn a_screenshot_goes_as_one_user_turn_with_text_and_image() {
        let v = super::image_message("data:image/jpeg;base64,AAAA", Some("Only question 2."));
        assert_eq!(v["role"], "user");
        assert_eq!(v["content"][0]["type"], "input_text");
        assert!(v["content"][0]["text"].as_str().unwrap().ends_with("Only question 2."));
        assert_eq!(v["content"][1], serde_json::json!({ "type": "input_image", "image_url": "data:image/jpeg;base64,AAAA" }));
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

#[cfg(test)]
mod instructions_tests {
    use super::{instructions, style_paragraph};

    /// Diagrams (mermaid sketches) are CloakView Pro: the free instructions must
    /// not mention them, or the model would draw sketches for free users too.
    #[test]
    fn only_the_pro_instructions_mention_diagrams() {
        for for_screen in [false, true] {
            let pro = instructions(for_screen, true, None, "balanced");
            assert!(pro.contains("mermaid"), "Pro instructions lost the diagram paragraph");
            let free = instructions(for_screen, false, None, "balanced");
            assert!(!free.contains("mermaid"), "free instructions must not offer diagrams");
            assert!(!free.contains("diagram"), "free instructions must not offer diagrams");
        }
    }

    #[test]
    fn free_and_pro_differ_only_by_the_diagram_paragraph() {
        assert!(instructions(false, true, None, "balanced").len() > instructions(false, false, None, "balanced").len());
        assert!(instructions(true, true, None, "balanced").len() > instructions(true, false, None, "balanced").len());
        assert!(instructions(false, false, None, "balanced").starts_with("You are a private copilot"));
        assert!(instructions(true, false, None, "balanced").starts_with("You are a private copilot"));
    }

    /// Personal context is CloakView Pro: it reaches the model only with a
    /// license, and blank notes are the same as none.
    #[test]
    fn personal_context_goes_out_only_for_pro() {
        let notes = "Senior Rust engineer, applying to fintech startups.";
        for for_screen in [false, true] {
            assert!(instructions(for_screen, true, Some(notes), "balanced").contains(notes));
            assert!(!instructions(for_screen, false, Some(notes), "balanced").contains(notes));
            assert!(!instructions(for_screen, true, Some("   "), "balanced").contains("own notes"));
            assert!(!instructions(for_screen, true, None, "balanced").contains("own notes"));
        }
    }

    /// Answer styles are CloakView Pro: the paragraph reaches the model only
    /// with a license, and `balanced` (or an unknown value) adds nothing.
    #[test]
    fn answer_styles_go_out_only_for_pro() {
        for (style, marker) in [
            ("interview", "STAR"),
            ("sales", "sales call"),
            ("technical", "deep-dive"),
            ("short", "three short sentences"),
            ("coding", "edge cases"),
        ] {
            for for_screen in [false, true] {
                let pro = instructions(for_screen, true, None, style);
                assert!(pro.contains(marker), "style {style} lost its paragraph");
                assert!(!instructions(for_screen, false, None, style).contains(marker));
            }
        }
        assert_eq!(style_paragraph("balanced"), "");
        assert_eq!(style_paragraph("something-else"), "");
        let balanced = instructions(false, true, None, "balanced");
        assert!(!balanced.contains("STAR"));
        assert!(!balanced.contains("sales call"));
    }
}
