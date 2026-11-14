# Draft copy updates — provider data-flow wording (NOT APPLIED)

Date: 2026-11-14. Status: **draft only — do not apply without review.**
Reason: the app now sends audio **directly to the AI provider using the user's
own key** (OpenAI at launch; Groq/Together built but gated until verified).
Any line implying "nothing leaves your device" or "only the query goes out" is
inaccurate: call audio is streamed to OpenAI for transcription, and question
text goes to OpenAI for answers. CloakView itself still runs no servers, holds
no accounts and stores nothing of yours.

Facts every draft is held to:

- Call audio is captured on the Mac and streamed to OpenAI (user's own key)
  for live transcription.
- Answer/suggestion requests (typed questions, transcript context, screen
  reads) go to OpenAI with the same key.
- CloakView has no servers; nothing passes through CloakView. No telemetry.
- Transcripts, notes and settings stay on the Mac.
- At launch the provider is OpenAI; more providers are in the works (true:
  the architecture is built and Groq/Together profiles are code-complete but
  hidden pending real-call verification — say "more providers coming" only
  where it fits, never as a promise of a date).
- When these are applied, `es.json` needs the same edits (locale parity test).

---

## 1. Landing — "How it works", step 2 ("It listens")

Current (`web/src/dictionaries/en.json`, `home.howItWorks.steps[1]`):

> During the call, CloakView captures the audio and streams it to OpenAI for
> live transcription, then detects when it's your turn to speak.

Draft (plainer about what leaves the device):

> During the call, CloakView captures the audio on your Mac and sends it
> straight to OpenAI — with your key, never through us — for live
> transcription, then detects when it's your turn to speak.

Step 1 ("Connect your key") is already accurate; no change proposed. Step 3
("It suggests") is accurate; no change.

Related line worth fixing in the same pass
(`home.features.items[1]` "Local-first"):

Current:

> No accounts, no telemetry, no backend. Transcripts and notes stay on your
> Mac; the audio goes only to OpenAI, with your key.

This one is already truthful; no change proposed.

---

## 2. FAQ Q1 — "Does CloakView record my calls?"

Current (`web/src/dictionaries/en.json`, `faq.topics[0].items[0]`):

> There is no CloakView backend and no telemetry. Audio is captured on your
> Mac and sent to OpenAI with your key, and the API requests go straight
> there. Check OpenAI's data handling policy before using the service.

Draft (answer the literal question first: no recording by us; then what does
leave the device):

> CloakView records nothing and uploads nothing to CloakView, because there
> is no CloakView backend: no servers, no accounts, no telemetry. During a
> call, audio is captured on your Mac and streamed directly to OpenAI — using
> your own API key — so it can be transcribed and answered in real time.
> Transcripts and notes stay on your Mac. If you want a provider other than
> OpenAI, more are in the works; check OpenAI's data-handling policy for how
> they treat API traffic.

Also on the home-page mini-FAQ (`home.faq.items[0]`, "Are you listening to
me?"):

Current:

> No. There is no CloakView server to listen from: audio is captured on your
> Mac and sent straight to OpenAI, with your own key, to transcribe it and
> suggest replies. Nothing passes through us.

Draft:

> No. CloakView runs no servers: audio is captured on your Mac and sent
> straight to OpenAI, with your own key, to transcribe it and suggest
> replies. Nothing passes through us — but be clear-eyed: the audio does
> leave your Mac, and it goes to OpenAI, not to us.

---

## 3. Product Hunt — maker comment

Current (in `Cloakview-Launch/cloakview launch copy.md`, maker comment
section):

> We run no servers. BYOK — you bring your own OpenAI API key. The model runs
> on your key, your context. We never see your audio or transcripts (they go
> to OpenAI with your key; notes stay on your Mac).

Replace that paragraph with:

> We run no servers and record nothing. BYOK — you bring your own OpenAI API
> key, and everything the AI does happens on your key, your account: the call
> audio is streamed from your Mac straight to OpenAI for transcription, and
> your questions go there for answers. Nothing passes through CloakView,
> because there is no CloakView to pass through. Notes and transcripts stay
> on your Mac.

Optional addition at the end of the comment (true, low-key):

> More providers beyond OpenAI are in the works — the architecture is built.

No other part of the maker comment asserts data flow; the "What it doesn't
do" list ("Touch your audio in our cloud ❌") remains accurate as written
(we have no cloud) — leave it.

---

## 4. Show HN post

Current (`Cloakview-Launch/cloakview community launch.md`, Part 4) has an
unresolved placeholder:

> [VERIFY AGAINST REAL ARCHITECTURE BEFORE POSTING: state exactly what
> captured data (audio/transcript/screen text) goes to OpenAI, what stays
> local, and whether anything is stored on disk.] Inference calls go to
> OpenAI using your own key; nothing passes through our servers because we
> run none.

Draft — replace the placeholder with:

> Exact data flow: (1) Call audio is captured with ScreenCaptureKit on your
> Mac and streamed over a WebSocket to OpenAI's realtime transcription API,
> using your own API key. (2) The transcript is kept in memory and on disk in
> a local SQLite database on your Mac; nothing is uploaded as a recording.
> (3) When you ask for a suggestion (or read the screen), the relevant
> transcript lines and your question — and the screenshot, for screen reads —
> go to OpenAI's API with the same key. (4) CloakView runs no servers,
> collects no telemetry and creates no account; the only third parties
> involved are OpenAI (always, with your key) and Lemon Squeezy (only if you
> buy Pro). Delete the app and its data folder and nothing of yours remains
> anywhere else.

Also in the same post, the "Current state" bullet "Bring Your Own API Key
(OpenAI)" can gain one clause:

> Bring Your Own API Key (OpenAI today; more providers in the works)
