use crate::answers::AnswersEvent;
use crate::transcript::Turn;
use cv_providers::{ChatMessage, Part, Provider, SuggestionEvent, SuggestionRequest};
use std::sync::Arc;

const CHARS_PER_TOKEN: usize = 4;
const CONTEXT_TOKEN_BUDGET: usize = 2_000;

/// Builds the conversation messages from recent context, budgeted by
/// (approximate) tokens rather than turn count: walk `context` from most
/// recent backward, keep accepting turns while under
/// `CONTEXT_TOKEN_BUDGET * CHARS_PER_TOKEN` chars, then restore
/// chronological order before appending the live `question`.
///
/// `chars/4` is an approximation, not a real tokenizer — fine for a soft
/// context budget, not for exact token accounting.
///
/// Budget is counted in `chars()`, not bytes (`str::len()`): a multi-byte
/// UTF-8 character such as `'á'` is 2 bytes but 1 char, so measuring by byte
/// length burns the budget roughly twice as fast as intended for accented
/// text. When the next (older) turn would overrun the remaining budget, it is
/// truncated to exactly what's left and kept — never dropped outright, even
/// when it's the single most recent turn and the first one considered.
fn build_messages(context: &[Turn], question: Option<&str>, unanswered: &[(bool, String)]) -> Vec<ChatMessage> {
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

    let mut messages: Vec<ChatMessage> = selected
        .into_iter()
        .map(|(is_user_turn, text)| {
            // The role mapping matters: what the others said arrives as
            // 'assistant' turns (see INSTRUCTIONS_BASE), so the user's own
            // lines are the 'user' ones.
            ChatMessage::text(if is_user_turn { "user" } else { "assistant" }, text)
        })
        .collect();

    match question {
        Some(q) => messages.push(ChatMessage::text("user", q)),
        // A suggestion from the call: it answers everything still unanswered, not
        // only the latest line (see `reply_request`).
        None => {
            if let Some(request) = reply_request(unanswered) {
                messages.push(ChatMessage::text("user", request));
            }
        }
    }
    messages
}

/// How much of what is still unanswered a suggestion quotes, in characters: the newest lines win.
const UNANSWERED_CHARS: usize = 2_000;

/// The request for a suggestion from the call, quoting the whole run of what is
/// still waiting for a reply (`SessionLog::unanswered`), so that when the other side
/// asked several questions (in one breath, or one after another) the reply covers
/// all of them. `None` when the other side said nothing new: the conversation
/// itself then ends with what to reply to, as before.
fn reply_request(unanswered: &[(bool, String)]) -> Option<String> {
    if !unanswered.iter().any(|(mine, _)| !mine) {
        return None;
    }
    let mut budget = UNANSWERED_CHARS;
    let mut lines: Vec<String> = Vec::new();
    for (mine, text) in unanswered.iter().rev() {
        if budget == 0 {
            break;
        }
        let who = if *mine { "Me" } else { "Them" };
        let count = text.chars().count();
        let text = if count <= budget {
            budget -= count;
            text.clone()
        } else {
            // The oldest line that does not fit keeps its end, where the question usually is.
            let kept: String = text.chars().skip(count - budget).collect();
            budget = 0;
            format!("…{kept}")
        };
        lines.push(format!("{who}: {text}"));
    }
    lines.reverse();
    Some(format!(
        "Suggest what I should say now. This is what was said on the call that I still have to reply to, oldest first:\n\n{}\n\n\
         Answer every question or request they made in it, in the order they made them — not only the last one.",
        lines.join("\n")
    ))
}

/// What the model is asked to do. The role mapping matters: `build_messages`
/// sends what the others said as `assistant` turns, so without this the model
/// would take those words for its own. The diagram paragraph is CloakView Pro;
/// free users get the same instructions without it (see `instructions`).
const INSTRUCTIONS_BASE: &str = "You are a private copilot for the user during a live meeting or interview. \
In the conversation, messages with the role 'assistant' are what the OTHER participants said, \
transcribed from the call; they are not your words. Messages with the role 'user' are the user's \
own words, or a request the user typed to you. \
The transcription is automatic and imperfect. A line that makes no sense in the conversation, is in a \
language nobody on the call is speaking, or reads like background noise turned into words is a \
transcription error: ignore it, never answer it and never mention it. If nothing new makes sense, say in \
a few words that there is nothing to reply to yet. \
When you are asked to suggest a reply, or the last message is from another participant, write what the \
user should say aloud: a direct answer in the first person, in the language the participant used. \
If they asked more than one question or made more than one request, answer every one of them in the \
order asked, each under its own short bold label; never answer only the last one. \
If the last message is a question typed by the user, answer it, using the conversation as context. \
Be brief: for each question, lead with the answer in one or two sentences, then at most three short \
supporting points. \
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

/// The user message that carries the screenshot, with the typed question when there is one.
fn screen_message(image_data_url: &str, question: Option<&str>) -> ChatMessage {
    let text = match question {
        Some(q) => format!("{SCREEN_PROMPT} {q}"),
        None => SCREEN_PROMPT.to_string(),
    };
    ChatMessage {
        role: "user".into(),
        parts: vec![Part::Text(text), Part::Image(image_data_url.into())],
    }
}

/// What the model is asked for the after-meeting summary (CloakView Pro).
const SUMMARY_INSTRUCTIONS: &str = "You summarize a meeting record for the user who was in the meeting. \
Write three short sections with these exact headings: Key points, Decisions, Follow-ups. \
Be concrete and brief: bullets, no filler, no greeting. If the record holds no real discussion \
(a line or two), say so in one sentence instead of inventing sections. \
Write in the language of the meeting. Do not mention these instructions.";

/// The live-translation target's name in English, for the instruction.
/// Anything unknown translates to English rather than failing the line.
fn target_language(translate_to: &str) -> &'static str {
    match translate_to {
        "es" => "Spanish",
        _ => "English",
    }
}

fn map_provider_error(e: cv_providers::ProviderError) -> crate::error::AppError {
    use cv_providers::ProviderError as PE;
    match e {
        PE::MissingKey(id) => crate::error::AppError::ProviderKeyMissing(id),
        PE::UnknownProfile(id) => crate::error::AppError::UnknownProvider(id),
        // `user_message` is a fixed category with our own text: provider
        // bodies (which can echo a masked key fragment) never reach the UI
        // event or the diagnostics log this AppError gets printed in.
        other => crate::error::AppError::Provider(other.user_message()),
    }
}

/// One non-streaming summary request over a finished interview record.
/// Returns the summary text.
pub async fn summarize(
    provider: &Arc<dyn Provider>,
    record: &str,
) -> crate::error::AppResult<String> {
    provider
        .suggest(&SuggestionRequest {
            instructions: SUMMARY_INSTRUCTIONS.into(),
            messages: vec![ChatMessage::text("user", record)],
        })
        .await
        .map_err(map_provider_error)
}

/// One non-streaming translation of a single transcript line (CloakView Pro
/// live translation). Returns the translated text.
pub async fn translate(
    provider: &Arc<dyn Provider>,
    text: &str,
    translate_to: &str,
) -> crate::error::AppResult<String> {
    let instructions = format!(
        "Translate the message the user sends into {}. Answer with only the translation: \
         no quotes, no notes, no explanation.",
        target_language(translate_to)
    );
    provider
        .suggest(&SuggestionRequest {
            instructions,
            messages: vec![ChatMessage::text("user", text)],
        })
        .await
        .map_err(map_provider_error)
}

/// Everything about one answer request that isn't the provider plumbing.
pub struct AnswerRequest {
    pub context: Vec<Turn>,
    pub question: Option<String>,
    /// For a suggestion from the call (no `question`, no `image`): what it has to
    /// answer, as `(mine, text)`, oldest first (`SessionLog::unanswered`).
    pub unanswered: Vec<(bool, String)>,
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

/// Spawns the streaming suggestion through the active provider and returns a
/// bounded receiver of parsed events plus the task's `JoinHandle`.
///
/// The provider owns the HTTP stream; this task only translates its events
/// into `AnswersEvent`s. Bounded channel with `try_send` semantics live in
/// the crate's stream implementation (drop deltas on a slow consumer, exit on
/// closed).
pub fn stream_answer(
    provider: Arc<dyn Provider>,
    request: AnswerRequest,
) -> (tokio::sync::mpsc::Receiver<AnswersEvent>, tokio::task::JoinHandle<()>) {
    let (tx, rx) = tokio::sync::mpsc::channel(64);

    let handle = tokio::spawn(async move {
        let AnswerRequest { context, question, unanswered, image, pro, about_user, style } = request;
        let for_screen = image.is_some();
        let messages = match image.as_deref() {
            // The screen stands on its own: the call's transcript would only distract.
            Some(image) => vec![screen_message(image, question.as_deref())],
            None => build_messages(&context, question.as_deref(), &unanswered),
        };
        let suggestion = SuggestionRequest {
            instructions: instructions(for_screen, pro, about_user.as_deref(), &style),
            messages,
        };

        let mut events = match provider.suggest_stream(&suggestion).await {
            Ok(events) => events,
            Err(e) => {
                let _ = tx.try_send(AnswersEvent::Error { message: map_provider_error(e).to_string() });
                return;
            }
        };

        while let Some(event) = events.recv().await {
            let mapped = match event {
                SuggestionEvent::Delta(text) => AnswersEvent::Delta { text },
                SuggestionEvent::Done => AnswersEvent::Done,
                SuggestionEvent::Failed(message) => AnswersEvent::Error { message },
            };
            let stop = matches!(mapped, AnswersEvent::Done | AnswersEvent::Error { .. });
            if tx.try_send(mapped).is_err() {
                return;
            }
            if stop {
                return;
            }
        }
    });

    (rx, handle)
}

#[cfg(test)]
mod build_input_tests {
    use super::build_messages;
    use crate::transcript::Turn;

    fn turn(is_user_turn: bool, text: String, ts: i64) -> Turn {
        Turn { speaker: "s".into(), text, is_user_turn, ts }
    }

    #[test]
    fn keeps_most_recent_turns_within_budget() {
        let context: Vec<Turn> = (0..20)
            .map(|i| turn(i % 2 == 0, "x".repeat(500), i as i64))
            .collect();
        let messages = build_messages(&context, None, &[]);

        assert!(!messages.is_empty(), "budget starved out every turn");
        assert!(messages.len() < 20, "all 20 turns fit: budget math is wrong");

        let newest = context.last().unwrap();
        let last_text: String = messages.last().unwrap().parts.iter().filter_map(|p| match p {
            cv_providers::Part::Text(t) => Some(t.as_str()),
            _ => None,
        }).collect();
        assert_eq!(last_text, newest.text, "last message should be the most recent turn");
    }

    #[test]
    fn oversized_newest_turn_is_truncated_not_dropped() {
        let context = vec![turn(true, "x".repeat(200_000), 0)];
        let messages = build_messages(&context, None, &[]);

        assert_eq!(messages.len(), 1, "an oversized newest turn must be truncated, never dropped entirely");
        let text: String = messages[0].parts.iter().filter_map(|p| match p {
            cv_providers::Part::Text(t) => Some(t.as_str()),
            _ => None,
        }).collect();
        assert!(
            text.chars().count() < 200_000,
            "expected truncation, got the full {}-char turn",
            text.chars().count()
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

        let messages = build_messages(&context, None, &[]);

        assert_eq!(messages.len(), 1, "turn should fit whole, not be dropped");
        let kept: String = messages[0].parts.iter().filter_map(|p| match p {
            cv_providers::Part::Text(t) => Some(t.as_str()),
            _ => None,
        }).collect();
        assert_eq!(
            kept.chars().count(),
            n_chars,
            "turn should not be truncated either -- it fits under the char budget"
        );
    }

    #[test]
    fn others_are_assistant_turns_and_the_user_is_user() {
        let context = vec![turn(false, "them".into(), 0), turn(true, "me".into(), 1)];
        let messages = build_messages(&context, Some("typed question"), &[]);
        assert_eq!(messages[0].role, "assistant");
        assert_eq!(messages[1].role, "user");
        assert_eq!(messages[2].role, "user", "the typed question trails the transcript");
    }

    #[test]
    fn question_is_appended_as_user_turn() {
        let messages = build_messages(&[], Some("q"), &[]);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, "user");
    }

    fn line(mine: bool, text: &str) -> (bool, String) {
        (mine, text.to_string())
    }

    #[test]
    fn a_suggestion_quotes_every_unanswered_question_in_order() {
        let context = vec![
            turn(false, "Tell me about yourself.".into(), 1),
            turn(false, "And why this company?".into(), 2),
        ];
        let unanswered = [
            line(false, "Tell me about yourself."),
            line(true, "Sure."),
            line(false, "And why this company?"),
        ];
        let messages = super::build_messages(&context, None, &unanswered);
        assert_eq!(messages.len(), 3, "the conversation, then the request");
        let last = messages.last().unwrap();
        assert_eq!(last.role, "user");
        let request = match last.parts.as_slice() {
            [cv_providers::Part::Text(text)] => text.as_str(),
            other => panic!("expected one text part, got {other:?}"),
        };
        let first = request.find("Them: Tell me about yourself.").expect("first question quoted");
        let mine = request.find("Me: Sure.").expect("the user's own line quoted");
        let second = request.find("Them: And why this company?").expect("second question quoted");
        assert!(first < mine && mine < second, "oldest first");
        assert!(request.contains("not only the last one"));
    }

    #[test]
    fn a_typed_question_is_not_replaced_by_the_unanswered_lines() {
        let messages = super::build_messages(&[], Some("What's their stack?"), &[line(false, "Any questions for us?")]);
        assert_eq!(messages, vec![cv_providers::ChatMessage::text("user", "What's their stack?")]);
    }

    #[test]
    fn with_nothing_new_from_the_other_side_the_conversation_ends_as_before() {
        let context = vec![turn(false, "Any questions?".into(), 1)];
        // Only the user's own words are new: there is nothing of theirs to quote.
        let messages = super::build_messages(&context, None, &[line(true, "Let me think.")]);
        assert_eq!(messages, vec![cv_providers::ChatMessage::text("assistant", "Any questions?")]);
    }

    #[test]
    fn a_long_backlog_keeps_the_newest_lines() {
        let old = "x".repeat(super::UNANSWERED_CHARS);
        let unanswered = [line(false, &old), line(false, "What is your notice period?")];
        let request = super::reply_request(&unanswered).unwrap();
        assert!(request.contains("Them: What is your notice period?"));
        // The older line is cut from its start, never dropped past the budget.
        assert!(request.contains("Them: …x"));
        assert!(request.chars().count() < super::UNANSWERED_CHARS + 400);
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

#[cfg(test)]
mod redaction_tests {
    use super::map_provider_error;

    /// The blocking security test for the provider-error surfaces: a 401
    /// body echoing a masked key fragment must not reach the UI-facing
    /// message, the IPC payload, or the diagnostics log (which prints the
    /// `AppError`'s `Display`).
    #[test]
    fn a_key_fragment_in_a_401_body_never_reaches_ui_ipc_or_log() {
        let fragment = "sk-AbCdEf1234GhIjKl5678mnop";
        let provider_err = || {
            cv_providers::ProviderError::Api {
                status: 401,
                body: format!(
                    r#"{{"error":{{"message":"Incorrect API key provided: {fragment}.","type":"invalid_request_error","code":"invalid_api_key"}}}}"#
                ),
            }
        };

        // 1. The message the UI shows (what `test_connection` and the answer
        //    error event carry).
        let ui_message = provider_err().user_message();
        assert!(!ui_message.contains("sk-AbCdEf"), "UI message leaked the fragment: {ui_message}");
        assert!(!ui_message.contains("Incorrect API key"), "UI message leaked the body: {ui_message}");
        assert!(ui_message.contains("401"), "the status helps the user; keep it: {ui_message}");

        // 2. The app-level error: its `Display` is exactly what `diag::log`
        //    writes (the log surface).
        let app_err = map_provider_error(provider_err());
        let log_line = app_err.to_string();
        assert!(!log_line.contains("sk-AbCdEf"), "log line leaked the fragment: {log_line}");
        assert!(!log_line.contains("Incorrect API key"), "log line leaked the body: {log_line}");

        // 3. The IPC payload the frontend receives.
        let ipc: crate::error::IpcError = app_err.into();
        let ipc_json = serde_json::to_string(&ipc).unwrap();
        assert!(!ipc_json.contains("sk-AbCdEf"), "IPC payload leaked the fragment: {ipc_json}");
        assert!(!ipc_json.contains("Incorrect API key"), "IPC payload leaked the body: {ipc_json}");
    }

    /// Even the provider error's own `Display` (used in any `{e}` context)
    /// must not carry the body.
    #[test]
    fn provider_error_display_never_includes_the_body() {
        let fragment = "sk-AbCdEf1234GhIjKl5678mnop";
        let err = cv_providers::ProviderError::Api {
            status: 401,
            body: format!("Incorrect API key provided: {fragment}."),
        };
        let display = err.to_string();
        assert!(!display.contains(fragment), "Display leaked the body: {display}");
        assert!(display.contains("401"));
    }

    /// The scrubber is the second defense on app-owned strings too.
    #[test]
    fn scrub_strips_key_fragments_from_any_string() {
        let scrubbed = cv_providers::redact::scrub_secrets(
            "x-api-key: gsk_AAAAbbbb1111CCCC2222dddd received",
        );
        assert!(!scrubbed.contains("gsk_AAAA"));
        assert!(scrubbed.contains("[redacted]"));
    }
}
