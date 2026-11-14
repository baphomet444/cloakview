# CloakView release checklist (manual, launch-blocking)

Date: 2026-11-14. Run every step on the Mac that will ship the build.
Record results inline; anything failed blocks the launch copy claim it
supports.

## 1. Notarization

`export APPLE_ID=<Apple ID> APPLE_PASSWORD=<app-specific password> APPLE_TEAM_ID=RRZ966WRYC && npm run tauri build`

(Tauri notarizes automatically when those variables are set; the signing
identity is already in `tauri.conf.json`. Alternative: APPLE_API_KEY /
APPLE_API_ISSUER / APPLE_API_KEY_PATH.)

- Expected: build log shows no "skipping app notarization" warning;
  `xcrun notarytool history` shows **Accepted** for the submission;
  `spctl -a -vv src-tauri/target/release/bundle/macos/CloakView.app`
  prints `accepted source=Notarized Developer ID`.
- Gatekeeper on a second Mac (or a fresh macOS user): double-clicking the
  DMG-installed app opens normally, no "cannot verify developer" dialog.

## 2. Guided functional test (fresh install)

1. Screen Recording **denied**: press Listen → a clear "screen recording
   permission required" message appears, no crash, no silent failure.
2. Grant Screen Recording (+ microphone when testing mic) in System
   Settings, restart the app → Listen connects (hub shows the listening
   state).
3. System audio: play a YouTube video, Listen → transcript lines appear
   tagged from the call.
4. Microphone (include-mic on, headphones recommended): speak → your lines
   appear tagged as yours, and do not trigger `turn-detected`.
5. Settings → paste a real OpenAI key → **Test connection** shows both
   sides passing.
6. Ask a question / trigger turn-detected → a streamed answer appears.
7. Wrong key → Test connection shows "the API key was rejected (HTTP 401)";
   open the diagnostics log and confirm **no fragment of the key** appears
   anywhere.

## 3. Upgrade over an existing 0.2.1 install

Install the new DMG over the existing app (do not uninstall first).
Expected: all settings preserved (language, theme, opacity, shortcuts,
keywords, user name, personal context, answer style, translate target);
**no key re-prompt** — the openai slot is the same `openai-api-key`
Keychain entry 0.2.1 wrote, so the stored key just works; Listen works on
the first try after upgrade.

## 4. Overlay invisibility matrix (launch copy claims all three apps)

The overlay's invisibility rests on one setting, applied to every CloakView
window in `src-tauri/src/windows_ext.rs:82`:

```rust
win.setSharingType(NSWindowSharingType::None); // via objc2-app-kit
```

There is **no macOS-version branch in this code path** — only a compile-time
`cfg(target_os = "macos")`; the app's floor is macOS 14.2
(`minimumSystemVersion`). Known open question (flagged in the launch doc):
on macOS 15+ ScreenCaptureKit capture may ignore `sharingType = .none` in
some apps — this matrix exists to answer it empirically. **Only
configurations with a PASS row may be claimed as "invisible" in copy.**

Protocol per row: join a real call from a **second account/device** as the
remote participant. Share as indicated while the CloakView overlay is open
and showing suggestions locally. Ask the remote participant (or record the
receiving side) whether the overlay or any of its content is visible.
Also run one row with the overlay over the shared window on purpose as a
positive control — the remote side SHOULD see it there (proves the test
can detect the overlay at all).

| macOS version | App (version) | Share mode | Overlay visible to participants? | Pass/Fail |
|---|---|---|---|---|
| e.g. 14.x | Zoom (ver) | single window (not the one with the overlay) |  |  |
| e.g. 14.x | Zoom (ver) | **entire screen** |  |  |
| e.g. 14.x | Google Meet (browser+ver) | single window |  |  |
| e.g. 14.x | Google Meet (browser+ver) | **entire screen** |  |  |
| e.g. 14.x | Microsoft Teams (ver) | single window |  |  |
| e.g. 14.x | Microsoft Teams (ver) | **entire screen** |  |  |
| (repeat on macOS 15.x if available) | … | … |  |  |

Positive control row (must be VISIBLE): same call, share the window the
overlay sits on → remote sees the overlay. If this control fails (remote
sees nothing even here), the test setup cannot detect the overlay and every
other row is invalid.

## 5. Copy gate

Apply `docs/provider-copy-drafts.md` (after review) with en/es parity, and
only publish invisibility claims for the rows that passed in step 4.
