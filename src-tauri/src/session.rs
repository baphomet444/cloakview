//! The record of one interview: what was heard and what was suggested, from the
//! moment listening starts until it stops.
//!
//! The record lives in memory only. When listening stops, `finish` closes it:
//! the app saves the suggested answers as a note and offers to write the whole
//! interview to a `.txt` file the user picks. Pure (no I/O, no clock of its own),
//! so it is unit-tested directly.

use chrono::{Local, TimeZone};

/// How much of the prompt a suggestion quotes, in characters.
const PROMPT_CHARS: usize = 200;

#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    /// A line of the call, as transcribed.
    Heard { ts: i64, text: String, mine: bool },
    /// A suggested answer, with the question (or the last thing heard) it answers.
    Suggestion { ts: i64, prompt: String, text: String },
}

/// The interview in progress.
#[derive(Debug, Default)]
pub struct SessionLog {
    started_ms: Option<i64>,
    entries: Vec<Entry>,
}

/// A closed interview, ready to be saved.
#[derive(Debug, Clone, PartialEq)]
pub struct FinishedSession {
    pub started_ms: i64,
    pub ended_ms: i64,
    pub entries: Vec<Entry>,
    /// The note the suggested answers were saved to, so the export does not repeat it.
    pub auto_note_id: Option<String>,
}

impl FinishedSession {
    pub fn heard_count(&self) -> usize {
        self.entries.iter().filter(|e| matches!(e, Entry::Heard { .. })).count()
    }

    pub fn suggestion_count(&self) -> usize {
        self.entries.iter().filter(|e| matches!(e, Entry::Suggestion { .. })).count()
    }
}

impl SessionLog {
    /// Starts recording. Starting again while a record is open keeps it: stopping
    /// and restarting listening in the middle of an interview is one interview.
    pub fn begin(&mut self, now_ms: i64) {
        self.started_ms.get_or_insert(now_ms);
    }

    pub fn is_open(&self) -> bool {
        self.started_ms.is_some()
    }

    /// Records a transcribed line, `mine` when it came from the user's microphone.
    /// Nothing is recorded outside a session.
    pub fn heard(&mut self, ts: i64, text: &str, mine: bool) {
        let text = text.trim();
        if self.is_open() && !text.is_empty() {
            self.entries.push(Entry::Heard { ts, text: text.to_string(), mine });
        }
    }

    /// The most recent thing the other side said, which an automatic suggestion answers.
    pub fn last_heard(&self) -> Option<&str> {
        self.entries.iter().rev().find_map(|e| match e {
            Entry::Heard { text, mine: false, .. } => Some(text.as_str()),
            _ => None,
        })
    }

    /// Records a suggested answer. Nothing is recorded outside a session.
    pub fn suggestion(&mut self, ts: i64, prompt: &str, text: &str) {
        let text = text.trim();
        if self.is_open() && !text.is_empty() {
            self.entries.push(Entry::Suggestion {
                ts,
                prompt: truncate(prompt.trim(), PROMPT_CHARS),
                text: text.to_string(),
            });
        }
    }

    /// Closes the record. `None` when nothing was recorded. Either way the log is
    /// empty and closed afterwards.
    pub fn finish(&mut self, now_ms: i64) -> Option<FinishedSession> {
        let started = self.started_ms.take()?;
        let entries = std::mem::take(&mut self.entries);
        if entries.is_empty() {
            return None;
        }
        Some(FinishedSession { started_ms: started, ended_ms: now_ms, entries, auto_note_id: None })
    }
}

fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let cut: String = text.chars().take(max_chars).collect();
        format!("{}…", cut.trim_end())
    }
}

/// `mm:ss`, or `h:mm:ss` from one hour on.
pub fn elapsed(ms: i64) -> String {
    let total = (ms.max(0) / 1000) as u64;
    let (h, m, s) = (total / 3600, (total / 60) % 60, total % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

fn local(ms: i64) -> chrono::DateTime<Local> {
    Local.timestamp_millis_opt(ms).single().unwrap_or_else(Local::now)
}

/// The title of the automatic note, and the base of the file name.
pub fn title(started_ms: i64) -> String {
    format!("Interview {}", local(started_ms).format("%Y-%m-%d %H:%M"))
}

/// A file name that is safe on every system: no colons or slashes.
pub fn file_name(started_ms: i64) -> String {
    format!("CloakView interview {}.txt", local(started_ms).format("%Y-%m-%d %H%M"))
}

/// The suggested answers as the body of a note: each with what it answers.
pub fn note_body(session: &FinishedSession) -> String {
    session
        .entries
        .iter()
        .filter_map(|e| match e {
            Entry::Suggestion { prompt, text, .. } if !prompt.is_empty() => Some(format!("Q: {prompt}\nA: {text}")),
            Entry::Suggestion { text, .. } => Some(format!("A: {text}")),
            Entry::Heard { .. } => None,
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// A note the user wrote during the interview.
#[derive(Debug, Clone, PartialEq)]
pub struct NoteLine {
    pub title: String,
    pub content: String,
}

/// The whole interview as plain text: the conversation in order, then the notes.
pub fn render_txt(session: &FinishedSession, notes: &[NoteLine]) -> String {
    let start = local(session.started_ms);
    let minutes = ((session.ended_ms - session.started_ms).max(0) / 60_000).max(1);
    let mut out = String::new();
    out.push_str("CloakView - interview record\n");
    out.push_str(&format!(
        "Date: {} to {} ({minutes} min)\n",
        start.format("%Y-%m-%d, %H:%M"),
        local(session.ended_ms).format("%H:%M"),
    ));
    out.push_str("\nCONVERSATION\n\n");
    for entry in &session.entries {
        match entry {
            Entry::Heard { ts, text, mine } => {
                let who = if *mine { "You said" } else { "They said" };
                out.push_str(&format!("[{}] {who}: {text}\n\n", elapsed(ts - session.started_ms)));
            }
            Entry::Suggestion { ts, prompt, text } => {
                let to = if prompt.is_empty() { String::new() } else { format!(" (to: \"{prompt}\")") };
                out.push_str(&format!("[{}] Suggested answer{to}:\n", elapsed(ts - session.started_ms)));
                for line in text.lines() {
                    out.push_str(&format!("    {line}\n"));
                }
                out.push('\n');
            }
        }
    }
    if !notes.is_empty() {
        out.push_str("NOTES\n\n");
        for note in notes {
            let title = note.title.trim();
            if !title.is_empty() {
                out.push_str(&format!("{title}\n"));
            }
            out.push_str(note.content.trim());
            out.push_str("\n\n");
        }
    }
    out.trim_end().to_string() + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> FinishedSession {
        FinishedSession {
            started_ms: 1_700_000_000_000,
            ended_ms: 1_700_000_000_000 + 38 * 60_000,
            entries: vec![
                Entry::Heard { ts: 1_700_000_012_000, text: "Tell me about your last project.".into(), mine: false },
                Entry::Heard { ts: 1_700_000_016_000, text: "Sure, one moment.".into(), mine: true },
                Entry::Suggestion {
                    ts: 1_700_000_020_000,
                    prompt: "Tell me about your last project.".into(),
                    text: "I led a rewrite.\nIt shipped on time.".into(),
                },
            ],
            auto_note_id: None,
        }
    }

    #[test]
    fn nothing_is_recorded_outside_a_session() {
        let mut log = SessionLog::default();
        log.heard(1, "hello", false);
        log.suggestion(2, "q", "a");
        log.begin(0);
        assert!(log.finish(10).is_none());
    }

    #[test]
    fn a_session_records_in_order_and_finishing_resets_it() {
        let mut log = SessionLog::default();
        log.begin(100);
        log.heard(110, "  Hello there  ", false);
        log.heard(111, "   ", false);
        log.heard(112, "Let me think.", true); // the user's own line is not what a suggestion answers
        assert_eq!(log.last_heard(), Some("Hello there"));
        log.suggestion(120, "Hello there", "Hi!");
        let done = log.finish(200).expect("something was recorded");
        assert_eq!((done.started_ms, done.ended_ms), (100, 200));
        assert_eq!((done.heard_count(), done.suggestion_count()), (2, 1));
        assert!(!log.is_open());
        assert!(log.finish(300).is_none());
    }

    #[test]
    fn restarting_listening_keeps_one_interview() {
        let mut log = SessionLog::default();
        log.begin(100);
        log.heard(110, "one", false);
        log.begin(500); // stop + start again
        log.heard(510, "two", false);
        assert_eq!(log.finish(600).unwrap().started_ms, 100);
    }

    #[test]
    fn long_prompts_are_cut() {
        let mut log = SessionLog::default();
        log.begin(0);
        log.suggestion(1, &"x".repeat(500), "answer");
        let Entry::Suggestion { prompt, .. } = &log.finish(2).unwrap().entries[0] else { panic!() };
        assert_eq!(prompt.chars().count(), PROMPT_CHARS + 1);
        assert!(prompt.ends_with('…'));
    }

    #[test]
    fn elapsed_time_is_mm_ss_then_h_mm_ss() {
        assert_eq!(elapsed(0), "00:00");
        assert_eq!(elapsed(65_000), "01:05");
        assert_eq!(elapsed(3_725_000), "1:02:05");
        assert_eq!(elapsed(-5), "00:00");
    }

    #[test]
    fn the_note_holds_each_answer_with_its_question() {
        assert_eq!(
            note_body(&sample()),
            "Q: Tell me about your last project.\nA: I led a rewrite.\nIt shipped on time."
        );
    }

    #[test]
    fn the_text_file_has_the_conversation_then_the_notes() {
        let notes = [NoteLine { title: "Follow up".into(), content: "Send the portfolio.\n".into() }];
        let txt = render_txt(&sample(), &notes);
        assert!(txt.starts_with("CloakView - interview record\nDate: "));
        assert!(txt.contains("(38 min)"));
        assert!(txt.contains("[00:12] They said: Tell me about your last project.\n"));
        assert!(txt.contains("[00:16] You said: Sure, one moment.\n"));
        assert!(txt.contains("[00:20] Suggested answer (to: \"Tell me about your last project.\"):\n    I led a rewrite.\n    It shipped on time.\n"));
        assert!(txt.contains("NOTES\n\nFollow up\nSend the portfolio.\n"));
        assert!(txt.ends_with("Send the portfolio.\n"));
        // Without notes there is no NOTES section.
        assert!(!render_txt(&sample(), &[]).contains("NOTES"));
    }

    #[test]
    fn the_file_name_is_safe_everywhere() {
        let name = file_name(1_700_000_000_000);
        assert!(name.starts_with("CloakView interview 20") && name.ends_with(".txt"));
        assert!(!name.contains([':', '/', '\\']));
    }
}
