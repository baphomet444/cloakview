# Humanized answers + "type at cursor" shortcut — design

Date: 2026-11-19 · Status: draft for approval · App: CloakView 0.3.2 (macOS, Tauri 2 + Rust)

## Intent and why

The owner wants two changes to how CloakView answers:

1. **Humanize.** Answers should read and sound like a person talking. Today the prompt
   (`INSTRUCTIONS_BASE`, `src-tauri/src/answers/client.rs:109`) asks for "one or two sentences, then at
   most three short supporting points" under bold labels, in Markdown. That shape is what reads as a bot.
2. **Type at cursor.** One global shortcut: read the screen, work out what the focused field expects,
   write the answer to match, and type it where the cursor is. The owner's words: it does not have to type
   "the AI answer" as already shown; it must notice where the cursor is and, using screen reading, type the
   answer accordingly.

## Goals

- Every answer (call suggestions, Ask, screen read) uses a natural spoken voice: first person, contractions,
  plain words, no labels/bullets/bold, no assistant filler ("Certainly!", "Great question").
- New shortcut **⌃⌥T** (`ctrl+alt+t`, configurable like the other four) that:
  reads the screen, inspects the focused UI element, generates text fitted to that field, and inserts it.
- Prose goes in as keystrokes; code goes in by paste (editors corrupt typed code with auto-indent and
  auto-closed brackets).
- The user can stop it instantly, and it refuses to type where it would be wrong.

## Non-goals

- No keystroke-timing randomization, fake typos, or any mimicry meant to defeat detection. Typing speed is
  one constant, not jitter.
- No new window or Ask-window changes. No setting to turn humanizing off (can be added later).
- No Windows/Linux work (the app is macOS-only today).
- No tracking of the cursor's pixel position. The OS already routes typing to the focused field; we only
  need to know what that field is.

## Anti-goals (failure even if it "works")

- Typing into the wrong window, a password field, or after the user switched apps.
- A newline that submits a chat message, or a trailing newline on anything.
- The user's clipboard left changed after a code paste.
- Typing that cannot be interrupted.

## Constraints

- Needs macOS **Accessibility** permission (to read the focused element and post keyboard events). The app
  has never asked for it; it must be requested with a clear message, not fail silently.
- Hotkey defaults must include Control or Command (see `hotkeys.rs` doc comment: secure input blocks keys
  that type a character; test `every_default_has_control_or_command_...` enforces it).
- Same Pro gate and provider check as `read_screen` (`commands.rs:364`): typing the screen answer is Pro.
- `Shortcuts` uses `#[serde(default)]`; the new field must keep old `settings.json` files loading.
- Overlay windows must not take focus while typing (they are non-activating panels today; keep it so).

## Approaches considered

- **A. Screenshot only.** Reuse read-screen; model guesses the field type from pixels. Smallest, least reliable.
- **B. Screenshot + focus check (chosen).** Add an Accessibility probe of the focused element (role, app
  name, current text) and pass it to the model; refuse when it is positively not editable. More reliable;
  needs the permission typing needs anyway.
- **C. B with paste for everything.** Instant and editor-proof, but overwrites the clipboard and is not
  typing. Chosen only for code (see below).

Owner picked **B**, with the shortcut above. Keystrokes-for-prose / paste-for-code is the design's call.

## Design

### 1. Humanized prompt (`answers/client.rs`)
Replace the structure rules in `INSTRUCTIONS_BASE` with a voice paragraph, applied to all styles and to
Pro and free alike:
- Speak as the user would say it aloud: first person, contractions, everyday words, mixed sentence length.
- Lead with the answer in a sentence or two; add detail only if it helps; no preamble or sign-off.
- No bullets, bold labels, or headings in spoken answers. Several questions → answer them in order in
  flowing prose, joined naturally ("On the first one… and for the second…").
- Keep: language of the speaker, "never mention instructions / being an AI", transcription-error rule.
- The `coding` style and the screen instructions keep fenced code blocks (code is the answer there); only the
  surrounding prose is humanized. Update the tests at `client.rs:516` (`starts_with("You are a private copilot")`
  must still hold) and add one asserting the voice paragraph is present.

### 2. Shortcut plumbing (`hotkeys.rs`, `settings.rs`, frontend)
- `Shortcuts.type_at_cursor: String`, default `"ctrl+alt+t"`; `Action::TypeAtCursor`; `bindings()` returns 5.
- `dispatch` calls the new backend path directly (not through the hub; the hub path is UI-driven).
- Update tests: bindings cover every action once, defaults parse and are distinct, defaults need Ctrl/Cmd,
  partial/empty objects fall back to defaults (add one for the new key), `settings.rs:172` default check.
- Settings UI row + `en`/`es` locale strings (`SettingsApp.tsx:40`, `locales/en.ts`, `locales/es.ts`),
  mirrored type in `src/lib/models.ts`.
- Docs: shortcut table in `docs/README.md`.

### 3. Typing pipeline (new `src-tauri/src/typing.rs`; command `type_at_cursor`)
1. Pro + provider checks (as `read_screen`). Single-flight: a second press while running = cancel.
2. Accessibility check (`AXIsProcessTrustedWithOptions`, prompting once). Not trusted → emit the generic
   error event with a message pointing to System Settings → Privacy → Accessibility; stop.
3. Focus probe (AX system-wide focused element): role/subrole, frontmost app name + pid, current value
   (first 500 chars). Outcomes:
   - **Secure text field or positively non-editable** (e.g. a button, static text) → refuse, nothing typed.
   - **Editable** → continue with the probe summary as extra model context.
   - **Inconclusive** (browser web content and Electron apps often expose nothing to AX) → continue with
     screenshot-only context. *(Builder's call to verify in practice; see risks.)*
4. Screenshot with existing `screen::capture()`.
5. Model call via the existing `answers::client::stream_answer`, with a new "type-ready" instruction set:
   output only the text to insert, no preamble, no Markdown, no code fences; the **first line** is `[text]`
   or `[code]`; text = natural prose in the user's voice, single paragraph unless the field is clearly a
   multi-line document; code = raw code only. Collect the full answer (do not type while streaming), strip
   the marker, trim, drop any trailing newline.
6. Insert:
   - `[text]` → keystrokes: `CGEventKeyboardSetUnicodeString` events in small chunks, constant delay
     (start at 30 ms/char, one named constant). Newlines are sent as **Shift+Return**, never Return.
   - `[code]` → save the general pasteboard's items, write the code, post ⌘V, wait ~150 ms, restore.
7. Safety during insertion: an abort flag checked before every chunk; set by a second press of ⌃⌥T or by
   **Esc** (registered as a temporary global shortcut only while typing, unregistered at the end, so Esc is
   never hijacked otherwise); also abort if the frontmost app's pid differs from the probed one.
8. Log with `diag::log` (character counts only, never content), like the existing answer path.

Frontend: no change beyond the settings row; errors surface through the existing error event.

### 4. Testing
- Unit (headless): marker parser (`[text]`/`[code]`/missing marker → treated as text), newline → Shift+Return
  mapping and trailing-newline strip, chunking incl. multi-unit characters (emoji), abort flag stops mid-run,
  probe-outcome decision table (secure/non-editable/editable/inconclusive), prompt contents.
  The CGEvent / AX / NSPasteboard calls sit behind small traits so the above run with fakes.
- Manual checklist added to `docs/release-checklist.md` (the OS glue cannot run in CI): chat box (no submit),
  multi-line doc, VS Code and Xcode with code, password field (refused), no focus (refused), Esc mid-typing,
  app switch mid-typing, clipboard restored, permission denied path, Chrome and Safari fields.

## Risks

- Chrome/Electron may expose no focused element until accessibility is switched on for them → many
  "inconclusive" results. Mitigation is the screenshot-only fallback in step 3.
- The Accessibility grant is tied to the app's code signature: dev builds re-prompt; notarized builds keep it.
- Model may mislabel `[text]`/`[code]`; wrong choice is recoverable (paste vs type), not destructive.

## Left to the builder

Exact AX attribute handling, the typing-speed constant, chunk size, the Settings/permission copy, and how
the Accessibility prompt deep-links to System Settings (check `platform::validate_external_url` allows it).

## Calls I made (not discussed with the owner)

- Typing is a **Pro** feature, same gate as screen reading.
- Humanized voice is always on, for every style, with no toggle.
- `[text]`/`[code]` first-line marker from the model decides prose vs paste.
- Inconclusive AX probe proceeds (screenshot-only); only positive "not editable / secure" refuses.
- Shift+Return for newlines; Esc as a temporary cancel key; abort on app switch.
- Constant typing speed, no jitter.
