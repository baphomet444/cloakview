# Humanized answers + type-at-cursor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make every AI answer sound spoken and human, and add a ⌃⌥T shortcut that reads the screen, inspects the focused field, and types (prose) or pastes (code) a fitted answer at the cursor.

**Architecture:** Pure logic (reply parsing, focus decision, insertion loop) lives in `typing/logic.rs` behind two small traits so it is unit-tested with fakes; the macOS glue (Accessibility probe, CGEvent keys, NSPasteboard) lives in `typing/mac.rs`; `typing/mod.rs` orchestrates and is called straight from the hotkey dispatcher. The model call is a new non-streaming function next to `summarize` in `answers/client.rs`.

**Tech Stack:** Rust (Tauri 2), `objc2` family (already deps: `objc2-app-kit`, `objc2-core-graphics`, `objc2-foundation`), `tauri-plugin-global-shortcut`, React/TS settings UI, vitest.

**Spec:** `docs/superpowers/specs/2026-11-19-type-at-cursor-design.md`

## Global Constraints

- Default shortcut is `ctrl+alt+t` (⌃⌥T); every default must include Control or Command (test `every_default_has_control_or_command_so_secure_input_does_not_block_it`).
- `Shortcuts` keeps `#[serde(default)]`; an old `settings.json` without the new key must load with the default.
- Typing is Pro: same `require_pro()` + `providers::resolve_active` gate as `read_screen` (`commands.rs:364`).
- Typing speed is one constant (`TYPE_DELAY_MS = 30`); no jitter, no simulated typos.
- Newlines are sent as Shift+Return, never Return; no trailing newline is ever sent.
- Code is inserted by paste (⌘V) and the user's pasteboard is restored afterwards; prose is typed.
- Esc is registered as a global shortcut only while typing and unregistered at the end.
- Logs (`diag::log`) carry character counts only, never content.
- macOS only; no new window; no setting to turn humanizing off.
- Run Rust tests with `cargo test --manifest-path src-tauri/Cargo.toml`; frontend with `npx vitest run` and `npx tsc --noEmit`.

## Review Focus

1. Emoji / accented / ZWJ text (e.g. `"café 👩‍💻"`) is typed intact: pieces re-join to the exact input. → Task 2.
2. Model returns an empty reply, only the marker, or only whitespace: nothing is typed, an error is reported, no Return is sent. → Task 2.
3. Model ignores the format: no marker → treated as text; code wrapped in ``` fences → fences removed before paste. → Task 2.
4. CRLF, runs of blank lines, trailing newlines → collapsed to single Shift+Return breaks; no trailing break. → Task 2.
5. ⌃⌥T pressed twice, or abort set before the first key: exactly one typist at a time, and an early abort types nothing. → Tasks 2 and 5.

---

### Task 1: Humanized voice in the answer prompt

**Files:**
- Modify: `src-tauri/src/answers/client.rs` (`INSTRUCTIONS_BASE` at line 109, tests module near line 516)

**Interfaces:**
- Produces: `INSTRUCTIONS_BASE` with the voice rules below; `instructions(...)` signature unchanged.

- [ ] **Step 1: Write the failing tests** in the existing `mod tests` of `client.rs`:
  - `spoken_voice_rules_are_in_every_call_instruction`: for `pro in [false, true]`, `instructions(false, pro, None, "balanced")` contains `"contractions"` and `"no bullets"`, and no longer contains `"short bold label"` or `"at most three short supporting points"`.
  - `humanizing_does_not_remove_code_blocks_for_the_coding_style`: `instructions(false, true, None, "coding")` still contains `"fenced code block"`.
  - Keep `free_and_pro_differ_only_by_the_diagram_paragraph` passing (`starts_with("You are a private copilot")`).
- [ ] **Step 2: Run** `cargo test --manifest-path src-tauri/Cargo.toml answers::client` — expect the two new tests to FAIL.
- [ ] **Step 3: Rewrite `INSTRUCTIONS_BASE`** keeping: the role-mapping sentences, the transcription-error rule, "language the participant used", and the Markdown/plain-text-math sentence limited to code. Replace the "each under its own short bold label" and "at most three short supporting points" rules with: speak as the user would say it aloud, first person, contractions, everyday words, mixed sentence length; lead with the answer in a sentence or two, add detail only if it helps; no preamble, sign-off or filler like "Certainly" / "Great question"; no bullets, headings or bold in spoken answers; several questions are answered in order as flowing prose ("On the first one… and for the second…"). Code blocks stay allowed for code. The wording must contain the literal phrases `contractions` and `no bullets`.
- [ ] **Step 4: Run** the same command — expect PASS (all `answers::client` tests, including the style tests).
- [ ] **Step 5: Commit** `feat: humanized spoken voice for every answer`

---

### Task 2: Pure typing logic

**Files:**
- Create: `src-tauri/src/typing/mod.rs` (only `pub mod logic;` for now), `src-tauri/src/typing/logic.rs`
- Modify: `src-tauri/src/lib.rs` (declare `mod typing;`)

**Interfaces:**
- Produces, all in `typing::logic`:
  - `pub enum Mode { Text, Code }`
  - `pub struct TypeReady { pub mode: Mode, pub text: String }`
  - `pub fn parse_type_ready(raw: &str) -> TypeReady` — reads an optional first-line marker `[text]` / `[code]` (case-insensitive, surrounding whitespace ignored); missing marker → `Text`; in `Code` mode strips a wrapping ``` fence (with optional language tag); normalizes `\r\n`/`\r` to `\n`, collapses runs of 3+ `\n` to 2, trims leading/trailing whitespace and newlines.
  - `pub enum Piece { Char(char), Newline }` and `pub fn pieces(text: &str) -> Vec<Piece>` — one `Char` per scalar (so an emoji is one piece), each `\n` is `Newline`.
  - `pub enum Focus { Secure, NotEditable, Editable { app: String, pid: i32, role: String, value: String }, Inconclusive { app: Option<String>, pid: Option<i32> } }`
  - `pub enum Decision { Refuse(&'static str), Proceed }` and `pub fn decide(focus: &Focus) -> Decision` — `Secure`/`NotEditable` refuse; `Editable`/`Inconclusive` proceed.
  - `pub trait Keyboard { fn type_char(&self, c: char) -> AppResult<()>; fn shift_return(&self) -> AppResult<()>; fn paste(&self, text: &str) -> AppResult<()>; fn frontmost_pid(&self) -> Option<i32>; }`
  - `pub enum Outcome { Done, Aborted }` and `pub fn insert(kb: &dyn Keyboard, ready: &TypeReady, expected_pid: Option<i32>, abort: &AtomicBool, delay: Duration) -> AppResult<Outcome>` — empty text → `Err(AppError::Audio("nothing to type"))` without touching `kb`; abort already set → `Ok(Aborted)` and nothing sent; `Code` → one `kb.paste(text)`; `Text` → per piece, check `abort` and (when `expected_pid` is `Some`) `kb.frontmost_pid() == expected_pid` before sending (mismatch → `Aborted`), then `type_char` or `shift_return`, then `std::thread::sleep(delay)`.
  - `pub const TYPE_DELAY_MS: u64 = 30;`

- [ ] **Step 1: Write the failing tests** in `logic.rs` `mod tests`, using a `FakeKeyboard` that records calls in a `Mutex<Vec<String>>` and a settable `frontmost_pid`:
  - `marker_selects_the_mode`: `"[code]\nfn a(){}"` → `Code`, text `"fn a(){}"`; `"[TEXT]\nhi"` → `Text`; `"no marker here"` → `Text`, text unchanged.
  - `code_fences_are_stripped_only_in_code_mode`: `"[code]\n```rust\nlet x = 1;\n```"` → `"let x = 1;"`; the same fence under `[text]` is left as typed.
  - `line_endings_and_blank_runs_are_normalized`: `"[text]\r\nA\r\n\r\n\r\n\r\nB\n\n"` → `"A\n\nB"`.
  - `empty_or_marker_only_replies_type_nothing`: `""`, `"[text]"`, `"[code]\n  \n"` → `insert` returns `Err`, fake records zero calls.
  - `emoji_and_accents_survive_pieces`: `"café 👩‍💻"` → re-joining `Char`s (and `Newline` as `\n`) equals the input.
  - `newlines_are_shift_return_and_never_return`: `"a\nb"` typed → calls `["a","SHIFT_RETURN","b"]`.
  - `abort_set_before_start_types_nothing`: abort `true` → `Ok(Aborted)`, zero calls.
  - `abort_mid_run_stops_between_pieces`: set abort from inside the fake after 2 chars of `"abcdef"` → `Aborted`, exactly 2 calls.
  - `app_switch_aborts`: fake pid changes from 7 to 8 after one char with `expected_pid = Some(7)` → `Aborted`, 1 call.
  - `code_mode_pastes_once`: `Code`, `"x\ny"` → exactly one `paste("x\ny")` call.
  - `decide_table`: `Secure`→Refuse, `NotEditable`→Refuse, `Editable{..}`→Proceed, `Inconclusive{..}`→Proceed.
  - Use `Duration::ZERO` in all `insert` tests.
- [ ] **Step 2: Run** `cargo test --manifest-path src-tauri/Cargo.toml typing::` — expect compile FAIL (items undefined).
- [ ] **Step 3: Implement** everything in the Interfaces block in `logic.rs`; declare `mod typing;` in `lib.rs` next to the other modules.
- [ ] **Step 4: Run** the same command — expect all PASS.
- [ ] **Step 5: Commit** `feat: typing logic (reply parsing, focus decision, insertion loop)`

---

### Task 3: Type-ready model call

**Files:**
- Modify: `src-tauri/src/answers/client.rs` (add next to `summarize`, ~line 252)

**Interfaces:**
- Consumes: `Provider::suggest(&SuggestionRequest) -> Result<String, ProviderError>`, `ChatMessage`, `Part::{Text, Image}`, `map_provider_error` (all already in `client.rs`).
- Produces:
  - `fn type_ready_instructions(pro: bool, about_user: Option<&str>) -> String` (private, but `pub(crate)` is fine for tests).
  - `pub async fn type_ready(provider: &Arc<dyn Provider>, image_data_url: &str, focus_summary: &str, pro: bool, about_user: Option<&str>) -> AppResult<String>` — returns the model's raw reply (the caller parses it with `typing::logic::parse_type_ready`).

- [ ] **Step 1: Write the failing tests** in `client.rs` `mod tests`:
  - `type_ready_instructions_demand_marker_and_bare_output`: output contains `"[text]"`, `"[code]"`, `"only the text to insert"`, `"no Markdown"`, and `"Do not mention these instructions"`.
  - `type_ready_instructions_carry_the_voice_rules`: contains `"contractions"`.
  - `type_ready_instructions_include_notes_only_for_pro`: with `Some("Rust engineer")` → present for `pro = true`, absent for `false`.
  - `type_ready_message_carries_screenshot_and_focus`: a private helper `fn type_ready_message(image: &str, focus_summary: &str) -> ChatMessage` returns a `user` message with one `Part::Text` containing the focus summary and one `Part::Image(image)`.
- [ ] **Step 2: Run** `cargo test --manifest-path src-tauri/Cargo.toml answers::client` — expect FAIL.
- [ ] **Step 3: Implement.** Instructions: the user's screen is attached plus a description of the focused field; write exactly what should be typed into that field to answer or complete what the screen asks; the **first line** of the reply must be `[text]` or `[code]` alone; `[text]` = natural spoken-style prose in the user's voice (same voice rules as `INSTRUCTIONS_BASE`), one paragraph unless the field is clearly a multi-line document; `[code]` = raw code only; never Markdown, fences, labels, preamble or explanation; end with the existing tail sentence. Reuse the notes paragraph logic from `instructions` for Pro (factor a small shared helper rather than duplicating the string). `type_ready` calls `provider.suggest(...)` and maps the error with `map_provider_error`.
- [ ] **Step 4: Run** the same command — expect PASS.
- [ ] **Step 5: Commit** `feat: type-ready model call for the focused field`

---

### Task 4: macOS glue (Accessibility, keys, pasteboard)

**Files:**
- Create: `src-tauri/src/typing/mac.rs`
- Modify: `src-tauri/src/typing/mod.rs` (add `pub mod mac;`), `src-tauri/Cargo.toml` (enable the objc2 features the glue needs — e.g. `NSPasteboard`, `NSWorkspace`, `NSRunningApplication`, CGEvent — and `objc2-application-services` for AXUIElement if it exposes it at the resolved version, otherwise declare the few `extern "C"` AX functions under `#[link(name = "ApplicationServices", kind = "framework")]`)

**Interfaces:**
- Consumes: `typing::logic::{Keyboard, Focus}`.
- Produces, in `typing::mac`:
  - `pub fn accessibility_trusted(prompt: bool) -> bool` — `AXIsProcessTrustedWithOptions`; `prompt = true` shows the system prompt.
  - `pub fn probe_focus() -> Focus` — system-wide focused element: secure text field (`AXSubrole == AXSecureTextField`) → `Secure`; role in `{AXTextField, AXTextArea, AXComboBox}` or a settable `AXValue` → `Editable` (value truncated to 500 chars, app name and pid from the frontmost application); a readable role that is none of those (button, static text, …) → `NotEditable`; any AX error or no element (browser web content, Electron) → `Inconclusive`.
  - `pub struct MacKeyboard;` implementing `Keyboard`: `type_char` posts a key-down/up pair carrying the character via `CGEventKeyboardSetUnicodeString` (UTF-16, so astral chars are one event); `shift_return` posts Return with the Shift flag; `paste` saves every item of `NSPasteboard.general`, writes the string, posts ⌘V, waits ~150 ms, restores the saved items (also on failure); `frontmost_pid` reads `NSWorkspace.frontmostApplication.processIdentifier`.
  - `pub fn describe(focus: &Focus) -> String` (pure; lives in `logic.rs` if you prefer, so it is testable) — a one-paragraph description for the model: app name, role, and the existing text if any, e.g. `Focused field: text area in "Slack"; it already contains: "…"`; `Inconclusive` → `Focused field: unknown (decide from the screenshot).`

- [ ] **Step 1: Write the failing test** `describe_mentions_app_role_and_existing_text_but_not_for_inconclusive` in `logic.rs` tests (or `mac.rs` if `describe` lives there): Editable → contains app, role and the value; Inconclusive → contains `"unknown"`.
- [ ] **Step 2: Run** `cargo test --manifest-path src-tauri/Cargo.toml typing::` — expect FAIL.
- [ ] **Step 3: Implement** `describe` and the `mac.rs` glue. Everything in `mac.rs` is `#[cfg(target_os = "macos")]`-free only if the crate is mac-only already (it is: `screen.rs` uses CoreGraphics unconditionally); follow `screen.rs`/`platform.rs` for style. These OS calls cannot run in CI; correctness is covered by the manual checklist in Task 6.
- [ ] **Step 4: Run** `cargo build --manifest-path src-tauri/Cargo.toml` and `cargo test --manifest-path src-tauri/Cargo.toml typing::` — expect build OK, tests PASS.
- [ ] **Step 5: Commit** `feat: macOS glue for focus probe, key injection and paste`

---

### Task 5: Orchestrator and the ⌃⌥T shortcut

**Files:**
- Modify: `src-tauri/src/typing/mod.rs` (add `run`), `src-tauri/src/hotkeys.rs` (field, action, bindings, dispatch arm, tests), `src-tauri/src/settings.rs` (test literals at ~163, ~172, ~249), `src/lib/models.ts:85-90`, `src/settings/SettingsApp.tsx:40`, `src/lib/locales/en.ts:77`, `src/lib/locales/es.ts:79`

**Interfaces:**
- Consumes: Task 2 (`parse_type_ready`, `decide`, `insert`, `Keyboard`, `TYPE_DELAY_MS`), Task 3 (`answers::client::type_ready`), Task 4 (`mac::{accessibility_trusted, probe_focus, MacKeyboard, describe}`), `screen::capture`, `AppState::{license, settings, is_pro}`, `providers::resolve_active`, `diag::log`, `events::ERROR_EVENT`.
- Produces:
  - `pub async fn typing::run(app: AppHandle)` — single-flight through a module-level `static RUNNING: Mutex<Option<Arc<AtomicBool>>>`: if one is running, set its abort flag and return (this is the "press again to cancel"); otherwise: Pro + provider checks → `accessibility_trusted(true)` else emit `ERROR_EVENT` with a message naming System Settings → Privacy & Security → Accessibility → `probe_focus` + `decide` (Refuse → emit `ERROR_EVENT` with the reason) → `screen::capture` in `spawn_blocking` → `type_ready` → `parse_type_ready` → register temporary `Escape` shortcut that sets the abort flag → `insert` in `spawn_blocking` with `expected_pid` from the probe (None when Inconclusive without pid) → unregister `Escape`, clear `RUNNING`, `diag::log` the char count. Every error path emits `ERROR_EVENT` (existing payload shape) and clears `RUNNING`; abort set while the model call is in flight also prevents any typing.
  - `Shortcuts.type_at_cursor: String` (default `"ctrl+alt+t"`, JSON key `typeAtCursor`), `Action::TypeAtCursor`, `bindings()` returning `[(Action, &str); 5]` with `TypeAtCursor` last; `dispatch` arm `Action::TypeAtCursor => { crate::typing::run(app.clone()).await; Ok(()) }`.
  - TS: `Shortcuts.typeAtCursor: string`; settings row `{ key: "typeAtCursor", labelKey: "settings.shortcutTypeAtCursor" }`; en: `"Read the screen and type the answer at the cursor"`; es: `"Leer la pantalla y escribir la respuesta en el cursor"`.

- [ ] **Step 1: Update/write the failing tests** in `hotkeys.rs` `mod tests`:
  - `an_empty_object_yields_the_default_shortcuts`: also `assert_eq!(s.type_at_cursor, "ctrl+alt+t")`.
  - `a_settings_file_without_the_new_key_gets_the_default`: `{"quickHide":"alt+h","pushToAsk":"x","toggleListen":"y","readScreen":"z"}` → `type_at_cursor == "ctrl+alt+t"`.
  - `serializes_with_camel_case_keys_and_roundtrips`: expected JSON gains `"typeAtCursor": "ctrl+alt+t"`.
  - `every_old_default_moves_...`: the literal JSON gains `"typeAtCursor": "ctrl+alt+t"` so the equality with `Shortcuts::default()` holds.
  - `bindings_cover_every_action_once`: expected vec ends with `Action::TypeAtCursor`.
  - The existing parse/distinct, Finder/Safari and Control-or-Command tests must pass unchanged for the new default.
  - In `settings.rs`, update the three literals/assertions that spell out the shortcuts (~163, 172, 249) to include `typeAtCursor`.
  - In `src/lib/ipc.test.ts`/settings tests, if any fixture builds a full `Shortcuts` object, add `typeAtCursor: "ctrl+alt+t"` so `npx tsc --noEmit` passes.
- [ ] **Step 2: Run** `cargo test --manifest-path src-tauri/Cargo.toml hotkeys settings` — expect FAIL.
- [ ] **Step 3: Implement** the Rust field/action/bindings/dispatch, `typing::run`, and the TS/locale changes above.
- [ ] **Step 4: Run** `cargo test --manifest-path src-tauri/Cargo.toml && cargo build --manifest-path src-tauri/Cargo.toml && cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`, then `npx vitest run && npx tsc --noEmit` — expect all PASS / no warnings.
- [ ] **Step 5: Commit** `feat: type-at-cursor shortcut (⌃⌥T)`

---

### Task 6: Docs, manual checklist, graph refresh

**Files:**
- Modify: `docs/README.md` (shortcut table), `docs/release-checklist.md`

- [ ] **Step 1: Add ⌃⌥T to the shortcut table in `docs/README.md`** with a two-line description (reads the screen, types at the cursor; Pro; Accessibility permission; Esc or ⌃⌥T cancels).
- [ ] **Step 2: Add a "Type at cursor" section to `docs/release-checklist.md`** with these manual checks, each a checkbox: chat box (answer typed, message not sent); multi-line document (line breaks appear, nothing submitted); VS Code and Xcode with code (pasted intact, clipboard restored); password field (refused, nothing typed); no focus / non-editable (refused); Esc mid-typing stops; ⌃⌥T mid-typing stops; switching apps mid-typing stops; Accessibility permission denied (clear message, no crash); Chrome and Safari text fields (works or falls back to screenshot-only); free-tier user (Pro prompt, nothing typed).
- [ ] **Step 3: Run** `graphify update .` (per project CLAUDE.md) and the full verification once more: `cargo test --manifest-path src-tauri/Cargo.toml && cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings && npx vitest run && npx tsc --noEmit` — expect all green.
- [ ] **Step 4: Commit** `docs: type-at-cursor shortcut and manual checklist`
