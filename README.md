# CloakView

Private AI meeting copilot for macOS: live transcription and suggested replies in an
overlay that stays out of the window you share. Bring your own OpenAI key, local-first,
no account and no backend.

- **Website and download:** https://getcloakview.com
- **Latest release:** https://github.com/baphomet444/cloakview/releases/latest
- **Documentation:** https://getcloakview.com/guide
- **Pricing:** https://getcloakview.com/pricing

macOS 14.2 or later on Apple Silicon. The app is free; CloakView Pro is a one-time
$99 payment, and you pay OpenAI directly for the usage you generate.

This repository holds the macOS app (React in `src/`, Rust in `src-tauri/`) and the
marketing site (`web/`, Next.js).

## Development notes

The app-level notes — known limitations, platform quirks and the reasoning behind
them — live in [docs/README.md](docs/README.md).
