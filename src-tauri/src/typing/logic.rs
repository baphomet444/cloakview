//! The part of "type at cursor" that needs no operating system: reading the
//! model's reply, deciding whether a focused field may be typed into, and the
//! insertion loop. The OS calls sit behind `Keyboard` (see `mac.rs`), so every
//! rule here runs in tests with a fake.

use crate::error::{AppError, AppResult};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Delay after every typed piece. One constant on purpose: a steady speed, no jitter.
/// How a person types: uneven gaps between keys, longer pauses after a word, a
/// sentence or a line, and now and then a pause to think. Self-contained
/// (xorshift) so no extra dependency is needed for a bit of jitter.
pub struct Rhythm {
    state: u64,
    enabled: bool,
}

impl Rhythm {
    pub fn seeded(seed: u64) -> Self {
        Self { state: seed | 1, enabled: true }
    }

    pub fn from_clock() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9);
        Self::seeded(nanos)
    }

    /// No waiting at all (tests).
    #[cfg(test)]
    pub fn instant() -> Self {
        Self { state: 1, enabled: false }
    }

    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// A value in `lo..=hi`.
    fn between(&mut self, lo: u64, hi: u64) -> u64 {
        lo + self.next() % (hi - lo + 1)
    }

    /// How long to wait after typing `c`.
    pub fn delay_for(&mut self, c: char) -> Duration {
        if !self.enabled {
            return Duration::ZERO;
        }
        // Average of two draws bunches the gaps around ~100 ms, like real typing.
        let mut ms = (self.between(55, 150) + self.between(55, 150)) / 2;
        ms += match c {
            '.' | '!' | '?' => self.between(180, 420),
            ',' | ';' | ':' => self.between(100, 220),
            '\n' => self.between(500, 1000),
            ' ' => self.between(0, 70),
            _ => 0,
        };
        // About one key in forty: a pause to think.
        if self.next().is_multiple_of(40) {
            ms += self.between(400, 1100);
        }
        Duration::from_millis(ms)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Text,
    Code,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeReady {
    pub mode: Mode,
    pub text: String,
}

/// Reads the model's reply: an optional first-line `[text]` / `[code]` marker
/// (no marker means text), then the text to insert, cleaned up.
pub fn parse_type_ready(raw: &str) -> TypeReady {
    let normalized = raw.replace("\r\n", "\n").replace('\r', "\n");
    let body = normalized.trim_start();
    let (first, rest) = match body.split_once('\n') {
        Some((first, rest)) => (first, rest),
        None => (body, ""),
    };
    let (mode, body) = match first.trim().to_ascii_lowercase().as_str() {
        "[code]" => (Mode::Code, rest),
        "[text]" => (Mode::Text, rest),
        _ => (Mode::Text, body),
    };
    let mut text = body.trim().to_string();
    if mode == Mode::Code {
        text = strip_fence(&text);
    }
    while text.contains("\n\n\n") {
        text = text.replace("\n\n\n", "\n\n");
    }
    TypeReady { mode, text: text.trim().to_string() }
}

/// ```lang … ``` around code, dropped; anything else is returned as is.
fn strip_fence(text: &str) -> String {
    let t = text.trim();
    if !t.starts_with("```") {
        return t.to_string();
    }
    let inner = t.split_once('\n').map(|(_, rest)| rest).unwrap_or("");
    let inner = inner.trim_end();
    inner.strip_suffix("```").unwrap_or(inner).trim().to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    Char(char),
    Newline,
}

/// One piece per character (an emoji is one piece, never split), each `\n` a `Newline`.
pub fn pieces(text: &str) -> Vec<Piece> {
    text.chars().map(|c| if c == '\n' { Piece::Newline } else { Piece::Char(c) }).collect()
}

/// What the Accessibility probe found at the cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Focus {
    Secure,
    NotEditable,
    Editable { app: String, pid: i32, role: String, value: String },
    /// Nothing readable (browser page content, Electron): decide from the screenshot.
    Inconclusive { app: Option<String>, pid: Option<i32> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Refuse(&'static str),
    Proceed,
}

pub fn decide(focus: &Focus) -> Decision {
    match focus {
        Focus::Secure => Decision::Refuse("The focused field is a password field; nothing was typed."),
        Focus::NotEditable => Decision::Refuse("The focused element is not a text field; nothing was typed."),
        Focus::Editable { .. } | Focus::Inconclusive { .. } => Decision::Proceed,
    }
}

/// One paragraph telling the model what the cursor is in.
pub fn describe(focus: &Focus) -> String {
    match focus {
        Focus::Editable { app, role, value, .. } => {
            let kind = match role.as_str() {
                "AXTextArea" => "text area",
                "AXTextField" => "text field",
                "AXComboBox" => "combo box",
                other => other,
            };
            let mut d = format!("Focused field: {kind} in \"{app}\".");
            if !value.trim().is_empty() {
                d.push_str(&format!(" It already contains: \"{value}\"."));
            }
            d
        }
        Focus::Inconclusive { app: Some(app), .. } => {
            format!("Focused field: unknown, in \"{app}\" (decide from the screenshot).")
        }
        Focus::Inconclusive { app: None, .. } => "Focused field: unknown (decide from the screenshot).".into(),
        // `decide` refuses these before any request is made; kept total for safety.
        Focus::Secure | Focus::NotEditable => "Focused field: not a text field.".into(),
    }
}

/// The few OS actions insertion needs.
pub trait Keyboard {
    fn type_char(&self, c: char) -> AppResult<()>;
    /// Shift+Return: a line break that does not submit a chat box.
    fn shift_return(&self) -> AppResult<()>;
    /// Paste `text` with ⌘V, leaving the pasteboard as it was.
    fn paste(&self, text: &str) -> AppResult<()>;
    fn frontmost_pid(&self) -> Option<i32>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Done,
    Aborted,
}

/// Inserts `ready` into the focused field. Stops (`Aborted`) before any piece if
/// `abort` is set or the frontmost app is no longer `expected_pid`.
pub fn insert(
    kb: &dyn Keyboard,
    ready: &TypeReady,
    expected_pid: Option<i32>,
    abort: &AtomicBool,
    rhythm: &mut Rhythm,
) -> AppResult<Outcome> {
    if ready.text.trim().is_empty() {
        return Err(AppError::Audio("nothing to type".into()));
    }
    let still_here = |kb: &dyn Keyboard| match expected_pid {
        Some(pid) => kb.frontmost_pid() == Some(pid),
        None => true,
    };
    if abort.load(Ordering::SeqCst) || !still_here(kb) {
        return Ok(Outcome::Aborted);
    }
    match ready.mode {
        Mode::Code => {
            kb.paste(&ready.text)?;
            Ok(Outcome::Done)
        }
        Mode::Text => {
            for piece in pieces(&ready.text) {
                if abort.load(Ordering::SeqCst) || !still_here(kb) {
                    return Ok(Outcome::Aborted);
                }
                let wait = match piece {
                    Piece::Char(c) => {
                        kb.type_char(c)?;
                        rhythm.delay_for(c)
                    }
                    Piece::Newline => {
                        kb.shift_return()?;
                        rhythm.delay_for('\n')
                    }
                };
                if !wait.is_zero() {
                    std::thread::sleep(wait);
                }
            }
            Ok(Outcome::Done)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Mutex;
    use std::time::Duration;

    type Hook<'a> = Box<dyn Fn(&FakeKeyboard<'a>) + 'a>;

    /// Records every call; `pid` is what `frontmost_pid` reports. `hook` runs
    /// right after the n-th call (to flip abort, or the frontmost pid).
    struct FakeKeyboard<'a> {
        calls: Mutex<Vec<String>>,
        pid: Mutex<Option<i32>>,
        hook: Option<(usize, Hook<'a>)>,
    }

    impl<'a> FakeKeyboard<'a> {
        fn new() -> Self {
            Self { calls: Mutex::new(vec![]), pid: Mutex::new(Some(7)), hook: None }
        }
        fn calls(&self) -> Vec<String> {
            self.calls.lock().unwrap().clone()
        }
        fn record(&self, s: String) {
            let n = {
                let mut c = self.calls.lock().unwrap();
                c.push(s);
                c.len()
            };
            if let Some((at, f)) = &self.hook {
                if n == *at {
                    f(self);
                }
            }
        }
    }

    impl Keyboard for FakeKeyboard<'_> {
        fn type_char(&self, c: char) -> crate::error::AppResult<()> {
            self.record(c.to_string());
            Ok(())
        }
        fn shift_return(&self) -> crate::error::AppResult<()> {
            self.record("SHIFT_RETURN".into());
            Ok(())
        }
        fn paste(&self, text: &str) -> crate::error::AppResult<()> {
            self.record(format!("PASTE:{text}"));
            Ok(())
        }
        fn frontmost_pid(&self) -> Option<i32> {
            *self.pid.lock().unwrap()
        }
    }

    fn ready(mode: Mode, text: &str) -> TypeReady {
        TypeReady { mode, text: text.into() }
    }

    #[test]
    fn marker_selects_the_mode() {
        let c = parse_type_ready("[code]\nfn a(){}");
        assert_eq!((c.mode, c.text.as_str()), (Mode::Code, "fn a(){}"));
        let t = parse_type_ready("  [TEXT]  \nhi");
        assert_eq!((t.mode, t.text.as_str()), (Mode::Text, "hi"));
        let n = parse_type_ready("no marker here");
        assert_eq!((n.mode, n.text.as_str()), (Mode::Text, "no marker here"));
    }

    #[test]
    fn code_fences_are_stripped_only_in_code_mode() {
        let code = parse_type_ready("[code]\n```rust\nlet x = 1;\n```");
        assert_eq!(code.text, "let x = 1;");
        let text = parse_type_ready("[text]\n```rust\nlet x = 1;\n```");
        assert!(text.text.contains("```"));
    }

    #[test]
    fn line_endings_and_blank_runs_are_normalized() {
        let t = parse_type_ready("[text]\r\nA\r\n\r\n\r\n\r\nB\n\n");
        assert_eq!(t.text, "A\n\nB");
    }

    #[test]
    fn empty_or_marker_only_replies_type_nothing() {
        for raw in ["", "[text]", "[code]\n  \n"] {
            let kb = FakeKeyboard::new();
            let r = parse_type_ready(raw);
            let abort = AtomicBool::new(false);
            assert!(insert(&kb, &r, None, &abort, &mut Rhythm::instant()).is_err(), "{raw:?}");
            assert!(kb.calls().is_empty(), "{raw:?} sent keys");
        }
    }

    #[test]
    fn emoji_and_accents_survive_pieces() {
        let input = "café 👩‍💻\nok";
        let joined: String = pieces(input)
            .into_iter()
            .map(|p| match p {
                Piece::Char(c) => c.to_string(),
                Piece::Newline => "\n".to_string(),
            })
            .collect();
        assert_eq!(joined, input);
    }

    #[test]
    fn newlines_are_shift_return_and_never_return() {
        let kb = FakeKeyboard::new();
        let abort = AtomicBool::new(false);
        let out = insert(&kb, &ready(Mode::Text, "a\nb"), None, &abort, &mut Rhythm::instant()).unwrap();
        assert_eq!(out, Outcome::Done);
        assert_eq!(kb.calls(), vec!["a", "SHIFT_RETURN", "b"]);
    }

    #[test]
    fn abort_set_before_start_types_nothing() {
        let kb = FakeKeyboard::new();
        let abort = AtomicBool::new(true);
        let out = insert(&kb, &ready(Mode::Text, "abc"), None, &abort, &mut Rhythm::instant()).unwrap();
        assert_eq!(out, Outcome::Aborted);
        assert!(kb.calls().is_empty());
    }

    #[test]
    fn abort_mid_run_stops_between_pieces() {
        let abort = AtomicBool::new(false);
        let mut kb = FakeKeyboard::new();
        kb.hook = Some((2, Box::new(|_| abort.store(true, Ordering::SeqCst))));
        let out = insert(&kb, &ready(Mode::Text, "abcdef"), None, &abort, &mut Rhythm::instant()).unwrap();
        assert_eq!(out, Outcome::Aborted);
        assert_eq!(kb.calls().len(), 2);
    }

    #[test]
    fn app_switch_aborts() {
        let abort = AtomicBool::new(false);
        let mut kb = FakeKeyboard::new();
        kb.hook = Some((1, Box::new(|k| *k.pid.lock().unwrap() = Some(8))));
        let out = insert(&kb, &ready(Mode::Text, "abcdef"), Some(7), &abort, &mut Rhythm::instant()).unwrap();
        assert_eq!(out, Outcome::Aborted);
        assert_eq!(kb.calls().len(), 1);
    }

    #[test]
    fn code_mode_pastes_once() {
        let kb = FakeKeyboard::new();
        let abort = AtomicBool::new(false);
        let out = insert(&kb, &ready(Mode::Code, "x\ny"), None, &abort, &mut Rhythm::instant()).unwrap();
        assert_eq!(out, Outcome::Done);
        assert_eq!(kb.calls(), vec!["PASTE:x\ny"]);
    }

    #[test]
    fn decide_table() {
        let editable = Focus::Editable {
            app: "Slack".into(),
            pid: 7,
            role: "AXTextArea".into(),
            value: String::new(),
        };
        assert!(matches!(decide(&Focus::Secure), Decision::Refuse(_)));
        assert!(matches!(decide(&Focus::NotEditable), Decision::Refuse(_)));
        assert!(matches!(decide(&editable), Decision::Proceed));
        assert!(matches!(decide(&Focus::Inconclusive { app: None, pid: None }), Decision::Proceed));
    }

    #[test]
    fn describe_mentions_app_role_and_existing_text_but_not_for_inconclusive() {
        let editable = Focus::Editable {
            app: "Slack".into(),
            pid: 7,
            role: "AXTextArea".into(),
            value: "hello wor".into(),
        };
        let d = describe(&editable);
        assert!(d.contains("Slack") && d.contains("text area") && d.contains("hello wor"), "{d}");
        let empty = Focus::Editable { app: "Notes".into(), pid: 1, role: "AXTextField".into(), value: String::new() };
        assert!(!describe(&empty).contains("already contains"));
        let unknown = describe(&Focus::Inconclusive { app: Some("Google Chrome".into()), pid: Some(3) });
        assert!(unknown.contains("unknown") && unknown.contains("Google Chrome"), "{unknown}");
        assert!(describe(&Focus::Inconclusive { app: None, pid: None }).contains("unknown"));
    }

    #[test]
    fn rhythm_varies_and_is_repeatable_per_seed() {
        let mut a = Rhythm::seeded(42);
        let mut b = Rhythm::seeded(42);
        let da: Vec<_> = "hello there".chars().map(|c| a.delay_for(c)).collect();
        let db: Vec<_> = "hello there".chars().map(|c| b.delay_for(c)).collect();
        assert_eq!(da, db);
        assert!(da.iter().any(|d| *d != da[0]), "a machine types at one fixed speed");
    }

    #[test]
    fn rhythm_letters_stay_in_a_human_range() {
        let mut r = Rhythm::seeded(7);
        for _ in 0..500 {
            let d = r.delay_for('e');
            assert!(d >= Duration::from_millis(45) && d <= Duration::from_millis(1500), "{d:?}");
        }
    }

    #[test]
    fn rhythm_pauses_longer_at_sentence_ends_and_line_breaks() {
        let mut r = Rhythm::seeded(3);
        let min_stop = (0..200).map(|_| r.delay_for('.')).min().unwrap();
        let min_nl = (0..200).map(|_| r.delay_for('\n')).min().unwrap();
        assert!(min_stop > Duration::from_millis(140), "{min_stop:?}");
        assert!(min_nl > min_stop, "{min_nl:?} vs {min_stop:?}");
    }

    #[test]
    fn rhythm_instant_never_waits() {
        let mut r = Rhythm::instant();
        assert!("a. b\n".chars().all(|c| r.delay_for(c).is_zero()));
    }
}
