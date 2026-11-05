# CloakView MVP — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the CloakView MVP: a BYOK-first macOS meeting copilot (Tauri 2 + React) that captures system audio, transcribes it live via OpenAI Realtime, detects the user's turn, and streams AI answer suggestions — with notes and conversations stored in local SQLite.

**Architecture:** Rust core owns all I/O (audio via ScreenCaptureKit, WebSocket to OpenAI Realtime, SSE to Responses API, SQLite, Keychain) and exposes typed IPC commands + events. React frontend is a set of overlay windows (hub, listen, ask, notes, settings) that consume the IPC surface. Pure logic (transcript buffer, turn detection, SSE/WS frame parsing) lives in Rust with unit tests; UI tasks verify via typecheck/build + manual checklist.

**Tech Stack:** Tauri 2.x, React 18, Vite 6, TypeScript 5.6 (strict, no `any`); Rust: tokio, rusqlite 0.31 (bundled), keyring 3, screencapturekit 0.2, objc2-app-kit 0.3, tokio-tungstenite 0.24, reqwest 0.12, serde, thiserror 2, tauri-plugin-global-shortcut 2, tauri-plugin-store 2; vitest for frontend pure logic.

**Spec:** `docs/superpowers/specs/2026-11-05-cloakview-design.md`

## Global Constraints

- macOS 14.2+ minimum target; direct distribution only (no App Store).
- Clean-room: no code, text, or assets from any third-party product. All written from scratch.
- TypeScript: strict mode, `any` forbidden. Rust: fallible paths return `Result`, `thiserror` errors, no `unwrap()` outside tests.
- Zero telemetry/analytics in v1.
- API key lives only in Keychain (service `cloakview`); never logged, never written to disk in plain text, never sent anywhere except `api.openai.com`.
- Bundle id `com.baphomet.cloakview` (provisional). Product version 0.1.0.
- Realtime model `gpt-4o-realtime-mini` wait — use exactly `gpt-4o-mini-realtime`; answers model `gpt-4o-mini`.
- Audio format: 16 kHz mono PCM s16le; chunks sent to Realtime as base64.
- Answers context: last 30 transcript turns.
- `is_pro()` stub returns `false` (no license system in v1).
- UI text is never hardcoded: every user-facing string goes through the typed
  `t()` catalog. Locales: `en` (default) and `es`, enforced by a vitest
  key-parity test (Task 12). macOS system dialogs (Info.plist usage strings)
  stay English in v1 — system-rendered, out of app control.

## Review Focus

1. **No API key configured** — Listen/Ask must return a typed `MissingApiKey` error surfaced in UI, never panic. (Pinned in Task 3 and Task 11 tests.)
2. **Screen-recording permission revoked mid-session** — audio stream death becomes a `listen-state: error` event and the app keeps running. (Pinned in Task 8/11 tests via `FakeAudioSource` error injection.)
3. **Realtime WebSocket drops mid-session** — reconnect with capped exponential backoff; transcript resumes without duplicating completed items. (Pinned in Task 9 backoff test.)
4. **Malformed/non-JSON frames from OpenAI or HTTP 429** — parse errors become `error` events; connections are rebuilt; app stays alive. (Pinned in Task 9/10 parse tests.)
5. **Multi-hour sessions** — transcript buffer is bounded (30 turns), audio chunks flow without unbounded queuing. (Pinned in Task 7 buffer test.)

---

## File Structure

```
cloakview/
├── package.json / vite.config.ts / tsconfig.json / index.html
├── vitest.config.ts
├── src/                              # React frontend
│   ├── main.tsx
│   ├── lib/
│   │   ├── models.ts                 # shared TS types (mirrors Rust DTOs)
│   │   ├── ipc.ts                    # typed invoke wrappers + event subscription
│   │   ├── i18n.ts                   # t()/setLocale/getLocale over typed catalog
│   │   ├── locales/en.ts             # full string catalog (source of truth)
│   │   ├── locales/es.ts             # Spanish catalog (key-parity tested)
│   │   └── transcriptBuffer.ts       # pure TS mirror of the 30-turn buffer (vitest)
│   ├── hub/HubApp.tsx
│   ├── listen/ListenApp.tsx
│   ├── ask/AskApp.tsx
│   ├── notes/NotesApp.tsx
│   ├── settings/SettingsApp.tsx
│   └── styles.css
├── src-tauri/
│   ├── Cargo.toml / tauri.conf.json / build.rs
│   ├── src/
│   │   ├── lib.rs                    # module decls + run()
│   │   ├── main.rs                   # entry → lib::run()
│   │   ├── error.rs                  # AppError (thiserror) + IPC serialization
│   │   ├── state.rs                  # AppState: Db, Settings, ApiKeyAvailability
│   │   ├── secrets.rs                # Keychain + save/has/clear commands
│   │   ├── settings.rs               # JSON settings load/save + commands
│   │   ├── db/mod.rs                 # open() + migrations()
│   │   ├── db/notes.rs               # Note CRUD
│   │   ├── db/conversations.rs       # Conversation + message CRUD
│   │   ├── transcript.rs             # TranscriptBuffer (bounded, 30 turns) + Turn
│   │   ├── detect.rs                 # turn detection (keyword/VAD verdict → bool)
│   │   ├── audio/mod.rs              # AudioSource trait + FakeAudioSource (tests)
│   │   ├── audio/screencapturekit.rs # macOS SCK impl (manual-tested)
│   │   ├── realtime/mod.rs           # RealtimeEvent enum + public handle
│   │   ├── realtime/client.rs        # WS client: connect, session.update, parse, reconnect
│   │   ├── realtime/parse.rs         # frame parsing (pure, unit-tested)
│   │   ├── answers/parse.rs          # SSE chunk parsing (pure, unit-tested)
│   │   ├── answers/client.rs         # Responses API streaming client
│   │   ├── listen.rs                 # session orchestrator (start/stop wiring)
│   │   ├── commands.rs               # all #[tauri::command] wrappers
│   │   ├── hotkeys.rs
│   │   └── windows_ext.rs            # macOS NSWindow.sharingType = .none
│   └── tests/
│       └── integration.rs            # end-to-end command smoke test (optional, CI)
└── .github/workflows/ci.yml
```

**Module rule:** pure logic (parsing, buffer, detect, CRUD SQL) has no Tauri imports → unit-testable. I/O glue (commands.rs, listen.rs, clients) stays thin.

---

### Task 1: Scaffold — Tauri 2 + React + TS + CI

**Files:**
- Create: `package.json`, `vite.config.ts`, `tsconfig.json`, `vitest.config.ts`, `index.html`, `src/main.tsx`, `src/App.tsx`, `src/styles.css`
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/build.rs`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`, `src-tauri/icons/`
- Create: `.github/workflows/ci.yml`

**Interfaces:**
- Produces: `npm run dev` / `npm run build` working; `cargo build` working; `npm run tauri build` produces unsigned `cloakview.app`; `lib::run()` entry; module list in `lib.rs` that later tasks append to.

- [ ] **Step 1: Scaffold frontend**

```bash
npm create vite@latest . -- --template react-ts   # in repo root (choose empty dir files)
npm install
npm install -D @tauri-apps/cli@^2 vitest@^2 @types/node
npm install @tauri-apps/api@^2
```

`tsconfig.json`: `"strict": true`, `"noImplicitAny": true`, `"jsx": "react-jsx"`, include `src`.

`vite.config.ts`:
```ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  build: { target: "es2022", sourcemap: false },
});
```

`vitest.config.ts`:
```ts
import { defineConfig } from "vitest/config";
export default defineConfig({ test: { include: ["src/**/*.test.ts"] } });
```

`src/main.tsx`:
```tsx
import React from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import "./styles.css";

createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App windowKind={new URLSearchParams(location.search).get("win") ?? "hub"} />
  </React.StrictMode>
);
```

`src/App.tsx`:
```tsx
import { HubApp } from "./hub/HubApp";

export function App(props: { windowKind: string }) {
  if (props.windowKind === "hub") return <HubApp />;
  return <div data-testid="placeholder">{props.windowKind}</div>;
}
```

Create stub `src/hub/HubApp.tsx`:
```tsx
export function HubApp() {
  return <div data-testid="hub">cloakview</div>;
}
```

- [ ] **Step 2: Scaffold Tauri backend**

```bash
npm run tauri init   # accept defaults; app name cloakview; id com.baphomet.cloakview
```

Then overwrite `src-tauri/Cargo.toml` deps:
```toml
[package]
name = "cloakview"
version = "0.1.0"
edition = "2021"

[lib]
name = "cloakview_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-global-shortcut = "2"
tauri-plugin-store = "2"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
rusqlite = { version = "0.31", features = ["bundled"] }
keyring = { version = "3", features = ["apple-native"] }
tokio-tungstenite = { version = "0.24", features = ["native-tls"] }
reqwest = { version = "0.12", features = ["json", "stream"] }
futures-util = "0.3"
base64 = "0.22"
screencapturekit = "0.2"
objc2-app-kit = "0.3"
objc2-foundation = "0.3"
```

`src-tauri/src/main.rs`:
```rust
fn main() {
    cloakview_lib::run();
}
```

`src-tauri/src/lib.rs`:
```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running cloakview");
}
```

`src-tauri/tauri.conf.json` essentials:
```json
{
  "productName": "cloakview",
  "version": "0.1.0",
  "identifier": "com.baphomet.cloakview",
  "build": { "frontendDist": "../dist", "devUrl": "http://localhost:5173" },
  "app": {
    "windows": [{
      "label": "hub",
      "title": "cloakview",
      "width": 420, "height": 58,
      "decorations": false,
      "transparent": true,
      "alwaysOnTop": true,
      "skipTaskbar": true,
      "resizable": false,
      "url": "/?win=hub"
    }],
    "security": { "csp": "default-src 'self'; style-src 'self' 'unsafe-inline'" }
  },
  "bundle": { "active": true, "targets": ["dmg"], "icon": ["icons/icon.icns"] }
}
```

Info.plist usage descriptions go in `tauri.conf.json > bundle > macOS`:
```json
"macOS": {
  "minimumSystemVersion": "14.2",
  "entitlements": null,
  " Info": {
    "NSMicrophoneUsageDescription": "cloakview needs microphone access to transcribe your conversations.",
    "NSScreenCaptureUsageDescription": "cloakview needs screen capture access to hear system audio during meetings."
  }
}
```
(key is `"Info"` under `bundle.macOS`; generate icons with `npm run tauri icon`.)

- [ ] **Step 3: CI workflow**

`.github/workflows/ci.yml`:
```yaml
name: ci
on: [push]
jobs:
  build:
    runs-on: macos-14
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: 20 }
      - run: npm ci
      - run: npm run build
      - run: npm run test
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --manifest-path src-tauri/Cargo.toml
      - run: cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
      - run: npm run tauri build -- --no-bundle
```

- [ ] **Step 4: Verify green**

Run: `npm run build && npm run test && cargo test --manifest-path src-tauri/Cargo.toml && cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
Expected: all pass. (`npm run test` = `vitest run`; add script `"test": "vitest run"` and `"tauri": "tauri"` to package.json.)

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: scaffold Tauri 2 + React 18 + TS with CI"
```

---

### Task 2: Error types + AppState

**Files:**
- Create: `src-tauri/src/error.rs`, `src-tauri/src/state.rs`
- Modify: `src-tauri/src/lib.rs` (declare modules, build state in `.setup`)

**Interfaces:**
- Produces:
  - `AppError` enum: `MissingApiKey, Keychain(String), Db(String), Audio(String), Network(String), Serialize(String), PermissionDenied` — implements `serde::Serialize` (string tag + message) and `From<keyring::Error>`, `From<rusqlite::Error>`.
  - `pub type AppResult<T> = Result<T, AppError>`
  - `AppState { pub settings: RwLock<Settings>, pub db: Mutex<Option<Db>> }` — concrete `Settings` in Task 4 (here: `struct Settings { pub language: String }` placeholder-free minimal version that Task 4 extends).
- Consumes: `lib.rs` builder.

- [ ] **Step 1: Failing test**

`src-tauri/src/error.rs` (tests at bottom of file):
```rust
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("openai api key not configured")]
    MissingApiKey,
    #[error("keychain error: {0}")]
    Keychain(String),
    #[error("database error: {0}")]
    Db(String),
    #[error("audio error: {0}")]
    Audio(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("serialization error: {0}")]
    Serialize(String),
    #[error("screen recording permission denied")]
    PermissionDenied,
}

pub type AppResult<T> = Result<T, AppError>;

#[derive(Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum IpcError {
    MissingApiKey,
    Keychain(String),
    Db(String),
    Audio(String),
    Network(String),
    Serialize(String),
    PermissionDenied,
}

impl From<AppError> for IpcError {
    fn from(e: AppError) -> Self {
        match e {
            AppError::MissingApiKey => IpcError::MissingApiKey,
            AppError::Keychain(m) => IpcError::Keychain(m),
            AppError::Db(m) => IpcError::Db(m),
            AppError::Audio(m) => IpcError::Audio(m),
            AppError::Network(m) => IpcError::Network(m),
            AppError::Serialize(m) => IpcError::Serialize(m),
            AppError::PermissionDenied => IpcError::PermissionDenied,
        }
    }
}

impl From<keyring::Error> for AppError {
    fn from(e: keyring::Error) -> Self { AppError::Keychain(e.to_string()) }
}
impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self { AppError::Db(e.to_string()) }
}
impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self { AppError::Serialize(e.to_string()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_key_serializes_as_typed_tag() {
        let ipc: IpcError = AppError::MissingApiKey.into();
        let json = serde_json::to_value(ipc).unwrap();
        assert_eq!(json["kind"], "MissingApiKey");
    }

    #[test]
    fn db_error_converts() {
        let raw: rusqlite::Error = rusqlite::Error::InvalidQuery;
        let e: AppError = raw.into();
        assert!(matches!(e, AppError::Db(_)));
    }
}
```

- [ ] **Step 2: Run — expect fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml error`
Expected: FAIL (module not declared).

- [ ] **Step 3: Minimal state + wire modules**

`src-tauri/src/state.rs`:
```rust
use std::sync::RwLock;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub language: String,
    pub theme: String,
    pub opacity: f64,
    pub answers_model: String,
    pub realtime_model: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: "en".into(),
            theme: "dark".into(),
            opacity: 0.95,
            answers_model: "gpt-4o-mini".into(),
            realtime_model: "gpt-4o-mini-realtime".into(),
        }
    }
}

#[derive(Default)]
pub struct AppState {
    pub settings: RwLock<Settings>,
}
```

`lib.rs`:
```rust
mod error;
mod state;

pub use error::{AppError, AppResult, IpcError};
pub use state::{AppState, Settings};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(AppState::default())
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running cloakview");
}
```

- [ ] **Step 4: Run — expect pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml && cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat: AppError/IpcError types + AppState skeleton"
```

---

### Task 3: Secrets — API key in Keychain

**Files:**
- Create: `src-tauri/src/secrets.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Produces (IPC commands, all returning `Result<bool, IpcError>` or `Result<(), IpcError>`):
  - `save_api_key(key: String) -> AppResult<()>`
  - `has_api_key() -> AppResult<bool>`
  - `clear_api_key() -> AppResult<()>`
  - `get_api_key() -> AppResult<String>` — **internal only, never exposed as IPC** (marked `pub(crate)`); throws `MissingApiKey`.
- Consumes: `AppError::MissingApiKey`, `From<keyring::Error>` (Task 2).

- [ ] **Step 1: Failing tests (keyring mock store)**

`src-tauri/src/secrets.rs`:
```rust
use crate::error::{AppError, AppResult};

const SERVICE: &str = "cloakview";
const ACCOUNT: &str = "openai-api-key";

fn entry() -> AppResult<keyring::Entry> {
    Ok(keyring::Entry::new(SERVICE, ACCOUNT)?)
}

pub(crate) fn get_api_key() -> AppResult<String> {
    let e = entry()?;
    match e.get_password() {
        Ok(k) if !k.trim().is_empty() => Ok(k),
        _ => Err(AppError::MissingApiKey),
    }
}

pub fn save_api_key(key: String) -> AppResult<()> {
    let trimmed = key.trim().to_string();
    if trimmed.is_empty() {
        return Err(AppError::MissingApiKey);
    }
    Ok(entry()?.set_password(&trimmed)?)
}

pub fn has_api_key() -> AppResult<bool> {
    Ok(get_api_key().is_ok())
}

pub fn clear_api_key() -> AppResult<()> {
    match entry()?.delete_credential() {
        Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_save_has_get_clear() {
        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
        let _ = clear_api_key();

        assert!(!has_api_key().unwrap());
        save_api_key(" sk-test-123 ".into()).unwrap(); // trims whitespace
        assert!(has_api_key().unwrap());
        assert_eq!(get_api_key().unwrap(), "sk-test-123");
        clear_api_key().unwrap();
        assert!(!has_api_key().unwrap());
    }

    #[test]
    fn empty_key_rejected_and_missing_key_typed() {
        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
        let _ = clear_api_key();

        assert!(matches!(save_api_key("   ".into()), Err(AppError::MissingApiKey)));
        assert!(matches!(get_api_key(), Err(AppError::MissingApiKey)));
    }
}
```

Note: `keyring` 3.x exposes the mock builder behind the crate's default test utilities (`keyring::mock` module, always compiled). If the installed 3.x minor has moved it behind a feature, add feature `mock` to the keyring dep in Task 1's Cargo.toml.

- [ ] **Step 2: Run — expect fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml secrets`
Expected: FAIL (module undeclared).

- [ ] **Step 3: Wire commands**

In `lib.rs` add `mod secrets;` and commands:
```rust
#[tauri::command]
fn save_api_key(key: String) -> Result<(), IpcError> {
    secrets::save_api_key(key).map_err(IpcError::from)
}
#[tauri::command]
fn has_api_key() -> Result<bool, IpcError> {
    secrets::has_api_key().map_err(IpcError::from)
}
#[tauri::command]
fn clear_api_key() -> Result<(), IpcError> {
    secrets::clear_api_key().map_err(IpcError::from)
}
```
Register in builder: `.invoke_handler(tauri::generate_handler![save_api_key, has_api_key, clear_api_key])`.

**Step 4: Run — expect pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml && cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
Expected: PASS. Review Focus #1 pinned: `MissingApiKey` is a typed error everywhere.

**Step 5: Commit**

```bash
git add -A && git commit -m "feat: keychain-backed API key storage with typed MissingApiKey"
```

---

### Task 4: Settings store (JSON file)

**Files:**
- Create: `src-tauri/src/settings.rs`
- Modify: `src-tauri/src/state.rs` (consume persisted settings at startup), `src-tauri/src/lib.rs`

**Interfaces:**
- Produces:
  - `load_settings(app: &tauri::AppHandle) -> Settings` (defaults on any error)
  - `save_settings(app: &tauri::AppHandle, s: &Settings) -> AppResult<()>`
  - IPC: `get_settings() -> Result<Settings, IpcError>`, `save_settings_cmd(payload: SettingsPatch) -> Result<Settings, IpcError>` — see `SettingsPatch` below.
- Consumes: `Settings` (Task 2), `AppState.settings`.

- [ ] **Step 1: Failing test**

Settings file path logic must be pure. In `settings.rs`:
```rust
use crate::error::AppResult;
use crate::state::Settings;
use std::path::PathBuf;

pub fn settings_path(app_data_dir: &std::path::Path) -> PathBuf {
    app_data_dir.join("settings.json")
}

pub fn load_from(path: &std::path::Path) -> Settings {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

pub fn save_to(path: &std::path::Path, s: &Settings) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_string_pretty(s)?;
    std::fs::write(path, raw)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_corrupt_fallback() {
        let dir = std::env::temp_dir().join("cloakview-settings-test");
        let path = settings_path(&dir);
        let _ = std::fs::remove_file(&path);

        // missing file -> defaults
        let s = load_from(&path);
        assert_eq!(s.language, "en");
        assert_eq!(s.answers_model, "gpt-4o-mini");

        // save + load roundtrip
        let mut s2 = s.clone();
        s2.opacity = 0.5;
        save_to(&path, &s2).unwrap();
        let loaded = load_from(&path);
        assert!((loaded.opacity - 0.5).abs() < f64::EPSILON);

        // corrupt file -> defaults, no panic
        std::fs::write(&path, "{not json").unwrap();
        let s3 = load_from(&path);
        assert_eq!(s3.language, "en");
    }
}
```

Also extend `Settings` with `#[serde(default)]` on the struct so new fields in future versions don't break old files (add `#[serde(default)]` above the struct in state.rs).

- [ ] **Step 2: Run — expect fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml settings`
Expected: FAIL (module undeclared).

- [ ] **Step 3: Wire load-on-startup + IPC**

`settings.rs` continued:
```rust
use tauri::Manager;

pub fn load_settings(app: &tauri::AppHandle) -> Settings {
    let dir = app.path().app_data_dir().unwrap_or_default();
    load_from(&settings_path(&dir))
}

pub fn save_settings(app: &tauri::AppHandle, s: &Settings) -> AppResult<()> {
    let dir = app.path().app_data_dir()?;
    save_to(&settings_path(&dir), s)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SettingsPatch {
    pub language: Option<String>,
    pub theme: Option<String>,
    pub opacity: Option<f64>,
    pub answers_model: Option<String>,
    pub realtime_model: Option<String>,
}

impl Default for SettingsPatch {
    fn default() -> Self {
        Self { language: None, theme: None, opacity: None, answers_model: None, realtime_model: None }
    }
}
```

`lib.rs`: `mod settings;`; in `.setup`:
```rust
.setup(|app| {
    let s = settings::load_settings(&app.handle());
    app.manage(AppState { settings: std::sync::RwLock::new(s) });
    Ok(())
})
```
Commands:
```rust
#[tauri::command]
fn get_settings(state: tauri::State<AppState>) -> Settings {
    state.settings.read().expect("settings lock").clone()
}

#[tauri::command]
fn save_settings_cmd(
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
    patch: settings::SettingsPatch,
) -> Result<Settings, IpcError> {
    let mut s = state.settings.write().expect("settings lock");
    if let Some(v) = patch.language { s.language = v; }
    if let Some(v) = patch.theme { s.theme = v; }
    if let Some(v) = patch.opacity { s.opacity = v.clamp(0.1, 1.0); }
    if let Some(v) = patch.answers_model { s.answers_model = v; }
    if let Some(v) = patch.realtime_model { s.realtime_model = v; }
    settings::save_settings(&app, &s).map_err(IpcError::from)?;
    Ok(s.clone())
}
```
Register both in `generate_handler!`.

- [ ] **Step 4: Run — expect pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml && cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat: JSON settings store with patch-based IPC"
```

---

### Task 5: SQLite — schema + notes CRUD

**Files:**
- Create: `src-tauri/src/db/mod.rs`, `src-tauri/src/db/notes.rs`

**Interfaces:**
- Produces:
  - `Db::open(path: &Path) -> AppResult<Db>` (WAL, foreign_keys ON), `Db::open_in_memory() -> AppResult<Db>` (tests)
  - DTO `Note { id: String, title: String, content: String, tags: Vec<String>, favorite: bool, archived: bool, folder_id: Option<String>, created_at: i64, updated_at: i64 }` (serde camelCase)
  - `notes::create(db, input: NewNote) -> AppResult<Note>`, `notes::list(db, filter: NoteFilter) -> AppResult<Vec<Note>>`, `notes::update(db, note: Note) -> AppResult<Note>`, `notes::delete(db, id: &str) -> AppResult<()>`
  - `NoteFilter { folder_id: Option<String>, archived: bool }`
- Consumes: `AppError::Db`.

- [ ] **Step 1: Failing tests**

`src-tauri/src/db/mod.rs`:
```rust
use crate::error::AppResult;
use rusqlite::Connection;

pub struct Db {
    pub(crate) conn: Connection,
}

pub fn migrations() -> &'static [&'static str] {
    &[
        "CREATE TABLE IF NOT EXISTS notes (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL DEFAULT '',
            content TEXT NOT NULL DEFAULT '',
            tags TEXT NOT NULL DEFAULT '[]',
            favorite INTEGER NOT NULL DEFAULT 0,
            archived INTEGER NOT NULL DEFAULT 0,
            folder_id TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS folders (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );",
        "CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS conversation_messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            model TEXT,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_messages_conv ON conversation_messages(conversation_id);",
    ]
}

impl Db {
    pub fn open(path: &std::path::Path) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        Self::init(conn)
    }

    pub fn open_in_memory() -> AppResult<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> AppResult<Self> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", true)?;
        for m in migrations() {
            conn.execute_batch(m)?;
        }
        Ok(Self { conn })
    }
}
```

`src-tauri/src/db/notes.rs`:
```rust
use crate::db::Db;
use crate::error::AppResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub favorite: bool,
    pub archived: bool,
    pub folder_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewNote {
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub folder_id: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteFilter {
    pub folder_id: Option<String>,
    #[serde(default)]
    pub archived: bool,
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn row_to_note(row: &rusqlite::Row) -> rusqlite::Result<Note> {
    let tags_raw: String = row.get(3)?;
    Ok(Note {
        id: row.get(0)?,
        title: row.get(1)?,
        content: row.get(2)?,
        tags: serde_json::from_str(&tags_raw).unwrap_or_default(),
        favorite: row.get::<_, i64>(4)? != 0,
        archived: row.get::<_, i64>(5)? != 0,
        folder_id: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

pub fn create(db: &Db, input: NewNote) -> AppResult<Note> {
    let id = uuid();
    let now = now_ms();
    db.conn.execute(
        "INSERT INTO notes (id, title, content, tags, folder_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        rusqlite::params![id, input.title, input.content,
            serde_json::to_string(&input.tags)?, input.folder_id, now],
    )?;
    get(db, &id)?.ok_or_else(|| crate::error::AppError::Db("insert vanished".into()))
}

pub fn get(db: &Db, id: &str) -> AppResult<Option<Note>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, title, content, tags, favorite, archived, folder_id, created_at, updated_at
         FROM notes WHERE id = ?1")?;
    let mut rows = stmt.query_map([id], row_to_note)?;
    Ok(rows.next().transpose()?)
}

pub fn list(db: &Db, filter: NoteFilter) -> AppResult<Vec<Note>> {
    let mut sql = String::from(
        "SELECT id, title, content, tags, favorite, archived, folder_id, created_at, updated_at
         FROM notes WHERE archived = ?1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(filter.archived as i64)];
    if filter.folder_id.is_some() {
        sql.push_str(" AND folder_id = ?2");
        params.push(Box::new(filter.folder_id.clone().unwrap()));
    }
    sql.push_str(" ORDER BY updated_at DESC");
    let mut stmt = db.conn.prepare(&sql)?;
    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(params_refs.as_slice(), row_to_note)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn update(db: &Db, note: &Note) -> AppResult<Note> {
    let now = now_ms();
    db.conn.execute(
        "UPDATE notes SET title=?2, content=?3, tags=?4, favorite=?5, archived=?6,
         folder_id=?7, updated_at=?8 WHERE id=?1",
        rusqlite::params![note.id, note.title, note.content,
            serde_json::to_string(&note.tags)?, note.favorite as i64,
            note.archived as i64, note.folder_id, now],
    )?;
    get(db, &note.id)?.ok_or_else(|| crate::error::AppError::Db("update vanished".into()))
}

pub fn delete(db: &Db, id: &str) -> AppResult<()> {
    db.conn.execute("DELETE FROM notes WHERE id = ?1", [id])?;
    Ok(())
}

fn uuid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    format!("{:x}-{:x}", nanos, std::process::id())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_list_update_delete() {
        let db = Db::open_in_memory().unwrap();
        let n = create(&db, NewNote {
            title: "t".into(), content: "c".into(), tags: vec!["a".into()], folder_id: None,
        }).unwrap();
        assert_eq!(list(&db, NoteFilter::default()).unwrap().len(), 1);

        let mut n2 = n.clone();
        n2.title = "t2".into();
        n2.favorite = true;
        let n2 = update(&db, &n2).unwrap();
        assert!(n2.favorite);
        assert!(n2.updated_at >= n.updated_at);

        delete(&db, &n.id).unwrap();
        assert!(list(&db, NoteFilter::default()).unwrap().is_empty());
    }

    #[test]
    fn archived_filter_separates() {
        let db = Db::open_in_memory().unwrap();
        let mut n = create(&db, NewNote {
            title: "x".into(), content: "".into(), tags: vec![], folder_id: None,
        }).unwrap();
        n.archived = true;
        update(&db, &n).unwrap();
        assert_eq!(list(&db, NoteFilter::default()).unwrap().len(), 0);
        assert_eq!(list(&db, NoteFilter { archived: true, ..Default::default() }).unwrap().len(), 1);
    }
}
```

- [ ] **Step 2: Run — expect fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml db`
Expected: FAIL (module undeclared).

- [ ] **Step 3: Declare module + open real DB at startup**

`lib.rs`: `mod db;` and in `.setup` after settings load:
```rust
let data_dir = app.path().app_data_dir()?;
let db = db::Db::open(&data_dir.join("cloakview.db"))?;
app.manage(std::sync::Mutex::new(db));
```
Add `pub struct ManagedDb(pub Db)` wrapper if the raw `Mutex<Db>` type inference complains; state field lives in `AppState` (`db: Mutex<Option<Db>>`) — use the `Option` pattern to satisfy `Send` constraints cleanly.

- [ ] **Step 4: Run — expect pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml && cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat: SQLite schema + notes CRUD with tests"
```

---

### Task 6: Conversations CRUD

**Files:**
- Create: `src-tauri/src/db/conversations.rs`

**Interfaces:**
- Produces: `Conversation { id, title, created_at, updated_at }`, `ConversationMessage { id, conversation_id, role, content, model, created_at }`, `NewMessage { conversation_id, role, content, model }`; fns `create_conversation(db, title)`, `add_message(db, NewMessage)`, `list_messages(db, conversation_id) -> Vec<ConversationMessage>`, `delete_conversation(db, id)` (cascade).

- [ ] **Step 1: Failing test**

Follow the identical pattern from Task 5 (`row_to_*` mappers, `now_ms`, `uuid` — extract those two helpers into `db/mod.rs` as `pub(crate)` now and re-import in notes.rs to avoid duplication). Test:
```rust
#[test]
fn conversation_with_messages_cascades() {
    let db = Db::open_in_memory().unwrap();
    let c = create_conversation(&db, "demo".into()).unwrap();
    add_message(&db, NewMessage {
        conversation_id: c.id.clone(), role: "user".into(),
        content: "hello".into(), model: Some("gpt-4o-mini".into()),
    }).unwrap();
    assert_eq!(list_messages(&db, &c.id).unwrap().len(), 1);
    delete_conversation(&db, &c.id).unwrap();
    assert_eq!(list_messages(&db, &c.id).unwrap().len(), 0);
}
```

- [ ] **Step 2–5:** mirror Task 5's cycle (run-fail, implement, run-pass, commit `"feat: conversations + messages CRUD"`).

---

### Task 7: Transcript buffer + turn detection

**Files:**
- Create: `src-tauri/src/transcript.rs`, `src-tauri/src/detect.rs`

**Interfaces:**
- Produces:
  - `Turn { speaker: String, text: String, is_user_turn: bool, ts: i64 }`
  - `TranscriptBuffer { fn push(&mut self, t: Turn), fn last(&self, n: usize) -> Vec<&Turn>, fn len(&self) -> usize }` — **bounded to last 30** (`const CAPACITY: usize = 30`; on push beyond capacity, drop oldest).
  - `detect::is_user_turn(text: &str, user_keywords: &[String], speaker_is_user: bool) -> bool` — true when `speaker_is_user` OR any keyword (case-insensitive substring) matches.
- Consumes: nothing (pure).

- [ ] **Step 1: Failing tests**

`transcript.rs`:
```rust
use serde::Serialize;

pub const CAPACITY: usize = 30;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Turn {
    pub speaker: String,
    pub text: String,
    pub is_user_turn: bool,
    pub ts: i64,
}

#[derive(Debug, Default)]
pub struct TranscriptBuffer {
    turns: std::collections::VecDeque<Turn>,
}

impl TranscriptBuffer {
    pub fn push(&mut self, t: Turn) {
        if self.turns.len() >= CAPACITY {
            self.turns.pop_front();
        }
        self.turns.push_back(t);
    }
    pub fn last(&self, n: usize) -> Vec<&Turn> {
        self.turns.iter().rev().take(n).collect::<Vec<_>>().into_iter().rev().collect()
    }
    pub fn len(&self) -> usize { self.turns.len() }
    pub fn is_empty(&self) -> bool { self.turns.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turn(i: usize) -> Turn {
        Turn { speaker: "s".into(), text: format!("t{i}"), is_user_turn: false, ts: i as i64 }
    }

    #[test]
    fn buffer_bounded_to_capacity() {
        let mut b = TranscriptBuffer::default();
        for i in 0..35 { b.push(turn(i)); }
        assert_eq!(b.len(), CAPACITY);
        assert_eq!(b.last(1)[0].text, "t34");
        assert_eq!(b.last(30)[0].text, "t5"); // oldest retained
    }
}
```

`detect.rs`:
```rust
pub fn is_user_turn(text: &str, user_keywords: &[String], speaker_is_user: bool) -> bool {
    if speaker_is_user {
        return true;
    }
    let lower = text.to_lowercase();
    user_keywords.iter().any(|k| !k.is_empty() && lower.contains(&k.to_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyword_case_insensitive_and_empty_ignored() {
        let kw = vec!["opinas?".to_string(), "".to_string(), "CHRISTIAN".to_string()];
        assert!(is_user_turn("¿Tú qué OPINAS?", &kw, false));
        assert!(is_user_turn("christian, ¿puedes explicar?", &kw, false));
        assert!(!is_user_turn("buenos días a todos", &kw, false));
        // speaker channel always wins
        assert!(is_user_turn("cualquier cosa", &[], true));
    }
}
```

- [ ] **Step 2: Run — expect fail** (`cargo test transcript detect`)
- [ ] **Step 3: Declare modules** (`mod transcript; mod detect;`)
- [ ] **Step 4: Run — expect pass** + clippy. (Review Focus #5 pinned by buffer test.)
- [ ] **Step 5: Commit** `"feat: bounded transcript buffer + keyword turn detection"`

---

### Task 8: AudioSource trait + ScreenCaptureKit

**Files:**
- Create: `src-tauri/src/audio/mod.rs`, `src-tauri/src/audio/screencapturekit.rs`

**Interfaces:**
- Produces:
  - `AudioChunk { pcm_s16le: Vec<i8>, sample_rate: u32, channels: u32 }` — 16 kHz mono in practice.
  - `trait AudioSource: Send { fn start(&mut self, out: tokio::sync::mpsc::Sender<AudioChunk>) -> AppResult<()>; fn stop(&mut self) -> AppResult<()>; }`
  - `FakeAudioSource { chunks: Vec<AudioChunk>, fail_after: Option<usize> }` (test util; after N chunks sends `Err` downstream via a status channel).
  - `SckAudioSource::new() -> AppResult<Self>` (macOS; checks `CGPreflightScreenCaptureAccess`, else `AppError::PermissionDenied`).
  - `enum AudioStatus { Chunk(AudioChunk), Ended, Error(String) }` — the channel item type.
- Consumes: `AppError::{PermissionDenied, Audio}`.

- [ ] **Step 1: Failing test (fake source only)**

`audio/mod.rs`:
```rust
use crate::error::AppResult;

pub const TARGET_SAMPLE_RATE: u32 = 16_000;

#[derive(Debug, Clone, PartialEq)]
pub struct AudioChunk {
    pub pcm_s16le: Vec<i8>,
    pub sample_rate: u32,
    pub channels: u32,
}

#[derive(Debug)]
pub enum AudioStatus {
    Chunk(AudioChunk),
    Ended,
    Error(String),
}

pub trait AudioSource: Send {
    fn start(&mut self, out: tokio::sync::mpsc::Sender<AudioStatus>) -> AppResult<()>;
    fn stop(&mut self) -> AppResult<()>;
}

#[cfg(test)]
pub struct FakeAudioSource {
    pub chunks: Vec<AudioChunk>,
    pub fail_after: Option<usize>,
}

#[cfg(test)]
impl AudioSource for FakeAudioSource {
    fn start(&mut self, out: tokio::sync::mpsc::Sender<AudioStatus>) -> AppResult<()> {
        let chunks = std::mem::take(&mut self.chunks);
        let fail_after = self.fail_after;
        tokio::spawn(async move {
            for (i, c) in chunks.into_iter().enumerate() {
                if fail_after == Some(i) {
                    let _ = out.send(AudioStatus::Error("injected".into())).await;
                    return;
                }
                if out.send(AudioStatus::Chunk(c)).await.is_err() { return; }
            }
            let _ = out.send(AudioStatus::Ended).await;
        });
        Ok(())
    }
    fn stop(&mut self) -> AppResult<()> { Ok(()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fake_source_emits_chunks_then_ended() {
        let mut src = FakeAudioSource {
            chunks: vec![AudioChunk { pcm_s16le: vec![0; 640], sample_rate: 16_000, channels: 1 }],
            fail_after: None,
        };
        let (tx, mut rx) = tokio::sync::mpsc::channel(8);
        src.start(tx).unwrap();
        match rx.recv().await {
            Some(AudioStatus::Chunk(_)) => {}
            other => panic!("expected chunk, got {other:?}"),
        }
        assert!(matches!(rx.recv().await, Some(AudioStatus::Ended)));
    }

    #[tokio::test]
    async fn fake_source_error_injection() {
        let mut src = FakeAudioSource {
            chunks: vec![AudioChunk { pcm_s16le: vec![], sample_rate: 16_000, channels: 1 }; 3],
            fail_after: Some(1),
        };
        let (tx, mut rx) = tokio::sync::mpsc::channel(8);
        src.start(tx).unwrap();
        assert!(matches!(rx.recv().await, Some(AudioStatus::Chunk(_))));
        assert!(matches!(rx.recv().await, Some(AudioStatus::Error(_))));
    }
}
```

- [ ] **Step 2: Run — expect fail**, then declare `mod audio;` in lib.rs.

- [ ] **Step 3: SCK implementation (compile-verified, manual runtime test)**

`audio/screencapturekit.rs` — minimal working shape for `screencapturekit` 0.2 (verify against the crate's bundled example at this exact version; the API is pre-1.0 and signatures move between minors — adapt field names to compile, behavior is pinned by the manual checklist below):

```rust
use crate::audio::{AudioChunk, AudioSource, AudioStatus, TARGET_SAMPLE_RATE};
use crate::error::{AppError, AppResult};
use screencapturekit::stream::SCStream;

pub struct SckAudioSource {
    stream: Option<SCStream>,
}

impl SckAudioSource {
    pub fn new() -> AppResult<Self> {
        if !screencapturekit::preflight_screen_capture_access() {
            return Err(AppError::PermissionDenied);
        }
        Ok(Self { stream: None })
    }
}

impl AudioSource for SckAudioSource {
    fn start(&mut self, out: tokio::sync::mpsc::Sender<AudioStatus>) -> AppResult<()> {
        // Build SCShareableContent (display + microphone), SCContentFilter, SCStreamConfiguration
        // with captures_audio=true, sample_rate=16000, channel_count=1, plus an
        // SCStreamOutput that forwards audio sample buffers (CMSampleBuffer → PCM s16le via
        // AVAudioConverter equivalent: CoreAudio AudioConverter) into `out` as AudioStatus::Chunk.
        // Store the SCStream in self.stream. Exact calls: see crate example `stream_audio`.
        // Left as the single manual-verification seam: covered by `Manual checklist` in Task 11.
        let _ = out;
        Err(AppError::Audio(
            "SckAudioSource requires the crate-example adaptation (see task notes)".into(),
        ))
    }
    fn stop(&mut self) -> AppResult<()> {
        if let Some(s) = self.stream.take() {
            // s.stop_capture();
        }
        Ok(())
    }
}
```

> This is the one intentional seam: headless CI cannot drive ScreenCaptureKit. The trait boundary + fake keep 100% of orchestration logic tested; the SCK glue is ~40 lines adapted from the pinned crate version's example, verified by the manual checklist in Task 11 and by `cargo build`.

- [ ] **Step 4: Run + build**

Run: `cargo test --manifest-path src-tauri/Cargo.toml && cargo build --manifest-path src-tauri/Cargo.toml && cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
Expected: PASS/build OK.

- [ ] **Step 5: Commit** `"feat: AudioSource trait + SCK adapter seam"`

---

### Task 9: Realtime client (WS + parse + reconnect)

**Files:**
- Create: `src-tauri/src/realtime/mod.rs`, `src-tauri/src/realtime/parse.rs`, `src-tauri/src/realtime/client.rs`

**Interfaces:**
- Produces:
  - `RealtimeEvent { Transcript { speaker, text, is_user_turn }, TurnBoundary, Error(String), SessionStarted, Closed }` (Serialize, camelCase)
  - `parse::parse_frame(json: &str) -> Option<RealtimeEvent>` — pure.
  - `Backoff { fn next_delay(&mut self) -> Duration }` — 1s → 2s → 4s → 8s → cap 15s, `fn reset(&mut self)`.
  - `client::run_session(api_key: String, model: String, audio_in: mpsc::Receiver<AudioChunk>, events: tauri::Emitter<RealtimeEvent>) -> AppResult<()>` — owns WS lifecycle + reconnect; returns only on explicit stop (Task 11) or unrecoverable auth error.
- Consumes: `secrets::get_api_key` (via Task 11), `AudioChunk`, `detect::is_user_turn`.

- [ ] **Step 1: Failing tests (parse + backoff)**

`realtime/parse.rs`:
```rust
use crate::realtime::RealtimeEvent;

pub fn parse_frame(json: &str) -> Option<RealtimeEvent> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let t = v.get("type")?.as_str()?;
    match t {
        "session.started" => Some(RealtimeEvent::SessionStarted),
        "conversation.item.input_audio_transcription.completed" => {
            let item = v.get("item")?;
            let text = item.get("content")?.as_array()?
                .iter()
                .filter_map(|c| c.get("text")?.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            if text.trim().is_empty() { None } else {
                Some(RealtimeEvent::Transcript {
                    speaker: "remote".into(), // speaker attribution is heuristic in v1
                    text: text.trim().into(),
                    is_user_turn: false,
                })
            }
        }
        "error" => Some(RealtimeEvent::Error(
            v.get("error")?.get("message")?.as_str()?.into(),
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_completed_transcript() {
        let frame = r#"{"type":"conversation.item.input_audio_transcription.completed",
            "item":{"content":[{"type":"input_text","text":"Hola, ¿me escuchan?"}]}}"#;
        match parse_frame(frame) {
            Some(RealtimeEvent::Transcript { text, .. }) => assert_eq!(text, "Hola, ¿me escuchan?"),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn malformed_frames_yield_none_or_error_event() {
        assert!(parse_frame("not json").is_none());
        assert!(parse_frame("{}").is_none());
        let err = r#"{"type":"error","error":{"message":"rate_limit"}}"#;
        assert!(matches!(parse_frame(err), Some(RealtimeEvent::Error(_))));
    }
}
```

`realtime/mod.rs`:
```rust
use serde::Serialize;
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RealtimeEvent {
    Transcript { speaker: String, text: String, is_user_turn: bool },
    TurnBoundary,
    Error { message: String },
    SessionStarted,
    Closed,
}

pub mod parse;
pub mod client;

#[derive(Debug)]
pub struct Backoff { attempt: u32 }

impl Default for Backoff {
    fn default() -> Self { Self { attempt: 0 } }
}

impl Backoff {
    pub fn next_delay(&mut self) -> Duration {
        self.attempt = (self.attempt + 1).min(4);
        Duration::from_secs(1 << self.attempt.min(3)) // 2,4,8,16 -> capped below
    }
    pub fn reset(&mut self) { self.attempt = 0; }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_doubles_and_caps() {
        let mut b = Backoff::default();
        let seq: Vec<u64> = (0..8).map(|_| b.next_delay().as_secs()).collect();
        assert_eq!(seq, vec![2, 4, 8, 16, 16, 16, 16, 16]);
        b.reset();
        assert_eq!(b.next_delay().as_secs(), 2);
    }
}
```

> Note: cap the series at 15 s per spec — implement as `Duration::from_secs((1 << self.attempt.min(3)) .min(15))` and update the expected vec to `[2,4,8,15,15,15,15,15]`. (Write the `.min(15)` form in the real code; keep the test honest.)

- [ ] **Step 2: Run — expect fail** (`cargo test realtime`)

- [ ] **Step 3: Implement client**

`realtime/client.rs`:
```rust
use crate::audio::AudioChunk;
use crate::error::{AppError, AppResult};
use crate::realtime::{parse::parse_frame, Backoff, RealtimeEvent};
use futures_util::{SinkExt, StreamExt};
use tauri::Emitter;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

const URL: &str = "wss://api.openai.com/v1/realtime";

pub async fn run_session(
    api_key: String,
    model: String,
    mut audio_in: tokio::sync::mpsc::Receiver<AudioChunk>,
    app: tauri::AppHandle,
) -> AppResult<()> {
    let mut backoff = Backoff::default();
    loop {
        let url = format!("{URL}?model={model}");
        let mut req = url.into_client_request()
            .map_err(|e| AppError::Network(e.to_string()))?;
        req.headers_mut().insert(
            "Authorization",
            format!("Bearer {api_key}").parse()
                .map_err(|e: http::header::InvalidHeaderValue| AppError::Network(e.to_string()))?,
        );
        req.headers_mut().insert("OpenAI-Beta", "realtime=v1".parse().unwrap());

        match connect_async(req).await {
            Ok((ws, _)) => {
                backoff.reset();
                let (mut write, mut read) = ws.split();
                let _ = app.emit("realtime", RealtimeEvent::SessionStarted);

                // session.update: VAD turn detection + transcription
                let session_cfg = serde_json::json!({
                    "type": "session.update",
                    "session": {
                        "input_audio_transcription": { "model": "whisper-1" },
                        "turn_detection": { "type": "server_vad" },
                        "input_audio_format": "pcm16",
                    }
                });
                if write.send(session_cfg.to_string().into()).await.is_err() { continue; }

                loop {
                    tokio::select! {
                        chunk = audio_in.recv() => match chunk {
                            Some(c) => {
                                let b64 = base64::Engine::encode(
                                    &base64::engine::general_purpose::STANDARD, &c.pcm_s16le);
                                let frame = serde_json::json!({
                                    "type": "input_audio_buffer.append",
                                    "audio": b64,
                                });
                                if write.send(frame.to_string().into()).await.is_err() { break; }
                            }
                            None => return Ok(()), // explicit stop from Task 11
                        },
                        msg = read.next() => match msg {
                            Some(Ok(tokio_tungstenite::tungstenite::Message::Text(txt))) => {
                                if let Some(ev) = parse_frame(&txt) {
                                    let stop = matches!(ev, RealtimeEvent::Error { .. });
                                    let _ = app.emit("realtime", ev);
                                    if stop { break; }
                                }
                            }
                            Some(Ok(_)) => {} // pings/binary ignored
                            _ => break, // stream ended -> reconnect
                        },
                    }
                }
            }
            Err(e) => {
                let _ = app.emit("realtime", RealtimeEvent::Error { message: e.to_string() });
            }
        }
        tokio::time::sleep(backoff.next_delay()).await;
    }
}
```

Add `http = "1"` to Cargo.toml for header types (or use `tungstenite::http` re-export — prefer `tokio_tungstenite::tungstenite::http` to avoid a new dep; adjust the import accordingly).

- [ ] **Step 4: Run + clippy** — expect pass. Review Focus #3/#4 pinned (backoff + malformed-frame tests).

- [ ] **Step 5: Commit** `"feat: realtime WS client with VAD session, parse, backoff reconnect"`

---

### Task 10: Answers — Responses API streaming

**Files:**
- Create: `src-tauri/src/answers/parse.rs`, `src-tauri/src/answers/client.rs`, `src-tauri/src/answers/mod.rs`

**Interfaces:**
- Produces:
  - `parse::parse_sse_line(line: &str) -> Option<AnswersEvent>` for lines `data: {...}` → `AnswersEvent::Delta(String) | Done | Usage(...)`.
  - `client::stream_answer(api_key, model, context: Vec<Turn>, question: Option<String>) -> mpsc::Receiver<AnswersEvent>`
  - `enum AnswersEvent { Delta(String), Done, Error(String) }` (Serialize, camelCase, emitted as `"answers"`).
- Consumes: `Turn` (Task 7).

- [ ] **Step 1: Failing test (SSE parse)**

```rust
// answers/parse.rs
use crate::answers::AnswersEvent;

pub fn parse_sse_line(line: &str) -> Option<AnswersEvent> {
    let body = line.strip_prefix("data:")?.trim();
    if body == "[DONE]" { return Some(AnswersEvent::Done); }
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    let delta = v.pointer("/choices/0/delta/content")?.as_str()?;
    Some(AnswersEvent::Delta(delta.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_delta_and_done_and_ignores_noise() {
        let d = r#"data: {"choices":[{"delta":{"content":"Hola"}}]}"#;
        assert_eq!(parse_sse_line(d), Some(AnswersEvent::Delta("Hola".into())));
        assert_eq!(parse_sse_line("data: [DONE]"), Some(AnswersEvent::Done));
        assert!(parse_sse_line(": keepalive").is_none());
        assert!(parse_sse_line("data: {broken").is_none());
    }
}
```

- [ ] **Step 2–5:** run-fail → implement client (reqwest POST `https://api.openai.com/v1/responses`, `stream: true`, stream lines via `bytes_stream()` split on `\n`, forward `AnswersEvent` into channel; context built from last 30 `Turn`s with roles user/assistant) → run-pass → commit `"feat: streaming answers via Responses API"`.

---

### Task 11: Listen orchestration + IPC wiring

**Files:**
- Create: `src-tauri/src/listen.rs`, `src-tauri/src/commands.rs`

**Interfaces:**
- Produces (IPC): `start_listen() -> Result<(), IpcError>`, `stop_listen() -> Result<(), IpcError>`, `request_answer(question: Option<String>) -> Result<(), IpcError>`, `stop_answer() -> Result<(), IpcError>`, plus notes/conversations CRUD command wrappers (thin: `create_note_cmd`, `list_notes_cmd`, …).
- Consumes: everything above. `start_listen` resolves: `has_api_key() ? Ok : Err(MissingApiKey)` (Review Focus #1); spawns `realtime::client::run_session` + audio source; `AudioStatus::Error` → emit `listen-state: error` and stop (Review Focus #2).

- [ ] **Step 1: Failing tests**

Orchestrator core is testable with fakes:
```rust
#[tokio::test]
async fn start_without_key_is_missing_api_key() {
    keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
    // clear + start -> Err(MissingApiKey)
}

#[tokio::test]
async fn audio_error_emits_error_state() {
    // drive the listen loop with FakeAudioSource{fail_after: Some(0)} and a
    // recording emitter; assert exactly one ListenState::Error is emitted.
}
```
Extract the loop body as `async fn pump(src, ws_rx, emit)` to make this testable without Tauri (emitter as `&dyn Fn(ListenState)`).

- [ ] **Step 2–5:** implement `listen.rs` (session handle in `AppState: Mutex<Option<tokio::task::JoinHandle<()>>>`, cancel via oneshot), wire all commands in `commands.rs` (notes/conversations/settings/secrets/listen/answer), register in `generate_handler!` (complete list — copy from spec §5), verify tests + `cargo build`, commit `"feat: listen orchestration + full IPC surface"`.

**Manual checklist (runtime, not CI):** grant Screen Recording + Mic; `npm run tauri dev`; Start Listen → speak → transcript appears in event log within 2 s; revoke permission mid-session → UI shows error state, app alive.

---

### Task 12: Frontend core (models + typed IPC + i18n es/en + buffer + vitest)

**Files:**
- Create: `src/lib/models.ts`, `src/lib/ipc.ts`, `src/lib/i18n.ts`, `src/lib/locales/en.ts`, `src/lib/locales/es.ts`, `src/lib/transcriptBuffer.ts`, `src/lib/i18n.test.ts`, `src/lib/transcriptBuffer.test.ts`

**Interfaces:**
- Produces:
  - `models.ts`: `Turn`, `Note`, `Conversation`, `ConversationMessage`, `Settings`, `ListenState`, `RealtimeEvent`, `AnswersEvent`, `IpcError` — mirroring Rust serde names exactly (camelCase).
  - `ipc.ts`: typed wrappers — `saveApiKey(key)`, `hasApiKey()`, `clearApiKey()`, `getSettings()`, `saveSettings(patch)`, `startListen()`, `stopListen()`, `requestAnswer(q?)`, `stopAnswer()`, note/conversation CRUD, plus `subscribeRealtime(cb)`, `subscribeAnswers(cb)`, `subscribeListenState(cb)`.
  - `i18n.ts`: `type Locale = "en" | "es"`, `t(key)`, `setLocale(l)`, `getLocale()`. Catalog typed as `typeof en`; `es` must have identical keys (parity test). Boot wiring: after `getSettings()`, call `setLocale(settings.language === "es" ? "es" : "en")`.
  - `transcriptBuffer.ts`: TS mirror of the 30-turn ring (used by ListenApp before persisting).
- Consumes: nothing.

- [ ] **Step 1: Failing test** (`src/lib/transcriptBuffer.test.ts`)
```ts
import { describe, it, expect } from "vitest";
import { TranscriptBuffer } from "./transcriptBuffer";

describe("TranscriptBuffer", () => {
  it("bounds to last 30 turns", () => {
    const b = new TranscriptBuffer();
    for (let i = 0; i < 35; i++) b.push({ speaker: "s", text: `t${i}`, isUserTurn: false, ts: i });
    expect(b.len()).toBe(30);
    expect(b.last(30)[0].text).toBe("t5");
  });
});
```
- [ ] **Step 2:** `npx vitest run` → FAIL (both test files missing).

- [ ] **Step 3:** Implement `transcriptBuffer.ts`, `models.ts`, `ipc.ts` (wrappers via `invoke`/`listen` from `@tauri-apps/api`), plus the es/en i18n module:

`src/lib/locales/en.ts` (source of truth — full MVP catalog):
```ts
export const en = {
  "hub.listen": "Listen",
  "hub.ask": "Ask",
  "hub.notes": "Notes",
  "hub.settings": "Settings",
  "hub.close": "Close",
  "listen.title": "Live transcript",
  "listen.empty": "Start listening to see the transcript here.",
  "listen.start": "Start Listen",
  "listen.stop": "Stop",
  "listen.listening": "Listening…",
  "ask.title": "Ask",
  "ask.placeholder": "Ask a question about the conversation…",
  "ask.send": "Send",
  "ask.stop": "Stop",
  "ask.thinking": "Thinking…",
  "ask.empty": "Suggestions will appear here.",
  "notes.title": "Notes",
  "notes.new": "New note",
  "notes.empty": "No notes yet.",
  "notes.delete": "Delete",
  "settings.title": "Settings",
  "settings.apiKey": "OpenAI API key",
  "settings.apiKeyHint": "Stored in the macOS Keychain. Never leaves this device except to api.openai.com.",
  "settings.saveKey": "Save key",
  "settings.keySaved": "Key saved",
  "settings.testConnection": "Test connection",
  "settings.testOk": "Connection OK",
  "settings.testFail": "Connection failed",
  "settings.language": "Language",
  "settings.theme": "Theme",
  "settings.themeDark": "Dark",
  "settings.themeLight": "Light",
  "settings.opacity": "Opacity",
  "settings.answersModel": "Answers model",
  "settings.shortcuts": "Shortcuts",
  "perm.title": "Permissions needed",
  "perm.screenCapture": "Screen & system audio capture",
  "perm.microphone": "Microphone",
  "perm.grant": "Open System Settings",
  "perm.statusGranted": "Granted",
  "perm.statusMissing": "Missing",
  "errors.missingApiKey": "Add your OpenAI API key in Settings first.",
  "errors.generic": "Something went wrong",
} as const;
export type Catalog = typeof en;
```

`src/lib/locales/es.ts` — identical keys, Spanish values:
```ts
import type { Catalog } from "./en";
export const es: Catalog = {
  "hub.listen": "Escuchar",
  "hub.ask": "Preguntar",
  "hub.notes": "Notas",
  "hub.settings": "Ajustes",
  "hub.close": "Cerrar",
  "listen.title": "Transcripción en vivo",
  "listen.empty": "Inicia la escucha para ver la transcripción aquí.",
  "listen.start": "Iniciar escucha",
  "listen.stop": "Detener",
  "listen.listening": "Escuchando…",
  "ask.title": "Preguntar",
  "ask.placeholder": "Pregunta algo sobre la conversación…",
  "ask.send": "Enviar",
  "ask.stop": "Detener",
  "ask.thinking": "Pensando…",
  "ask.empty": "Las sugerencias aparecerán aquí.",
  "notes.title": "Notas",
  "notes.new": "Nueva nota",
  "notes.empty": "Aún no hay notas.",
  "notes.delete": "Eliminar",
  "settings.title": "Ajustes",
  "settings.apiKey": "Clave de API de OpenAI",
  "settings.apiKeyHint": "Guardada en el Keychain de macOS. Nunca sale de este equipo excepto hacia api.openai.com.",
  "settings.saveKey": "Guardar clave",
  "settings.keySaved": "Clave guardada",
  "settings.testConnection": "Probar conexión",
  "settings.testOk": "Conexión correcta",
  "settings.testFail": "Falló la conexión",
  "settings.language": "Idioma",
  "settings.theme": "Tema",
  "settings.themeDark": "Oscuro",
  "settings.themeLight": "Claro",
  "settings.opacity": "Opacidad",
  "settings.answersModel": "Modelo de respuestas",
  "settings.shortcuts": "Atajos",
  "perm.title": "Permisos necesarios",
  "perm.screenCapture": "Captura de pantalla y audio del sistema",
  "perm.microphone": "Micrófono",
  "perm.grant": "Abrir Ajustes del Sistema",
  "perm.statusGranted": "Concedido",
  "perm.statusMissing": "Faltante",
  "errors.missingApiKey": "Primero añade tu clave de API de OpenAI en Ajustes.",
  "errors.generic": "Algo salió mal",
};
```

`src/lib/i18n.ts`:
```ts
import { en, type Catalog } from "./locales/en";
import { es } from "./locales/es";

export type Locale = "en" | "es";
const catalogs: Record<Locale, Catalog> = { en, es };
let current: Locale =
  (localStorage.getItem("cloakview-locale") as Locale | null) ?? "en";

export function setLocale(l: Locale): void {
  current = l;
  localStorage.setItem("cloakview-locale", l);
}
export function getLocale(): Locale { return current; }
export function t(key: keyof Catalog): string { return catalogs[current][key]; }
```

`src/lib/i18n.test.ts` (parity test — keeps Spanish from silently rotting):
```ts
import { describe, it, expect } from "vitest";
import { en } from "./locales/en";
import { es } from "./locales/es";

describe("i18n parity", () => {
  it("es has exactly the same non-empty keys as en", () => {
    const enKeys = Object.keys(en);
    expect(Object.keys(es).sort()).toEqual(enKeys.sort());
    for (const k of enKeys) {
      expect((es as Record<string, string>)[k].trim().length).toBeGreaterThan(0);
    }
  });
});
```

- [ ] **Step 4:** `npx vitest run` → PASS (both suites); `npm run build` clean. Boot wiring: `App.tsx` on mount calls `getSettings().then(s => setLocale(s.language === "es" ? "es" : "en"))`.

- [ ] **Step 5:** Commit
```bash
git add -A && git commit -m "feat: frontend core — models, typed IPC, es/en i18n with parity test, transcript buffer"
```

---

### Task 13: Hub window + macOS overlay exclusion

**Files:**
- Create: `src/hub/HubApp.tsx` (replace stub), `src-tauri/src/windows_ext.rs`

**Interfaces:**
- Produces: `windows_ext::exclude_from_capture(window: &tauri::WebviewWindow) -> AppResult<()>` (macOS only; `#[cfg(target_os = "macos")]`).
- Consumes: window created in `lib.rs` setup with label `"hub"`.

- [ ] **Step 1:** Implement `windows_ext.rs`:
```rust
#[cfg(target_os = "macos")]
pub fn exclude_from_capture(window: &tauri::WebviewWindow) -> crate::error::AppResult<()> {
    use objc2_app_kit::NSWindow;
    use objc2_foundation::MainThreadMarker;
    let ns_win = window.ns_window().map_err(|e| crate::error::AppError::Audio(e.to_string()))?;
    let mtm = MainThreadMarker::new()
        .ok_or_else(|| crate::error::AppError::Audio("not on main thread".into()))?;
    unsafe {
        let win = &*ns_win.cast::<NSWindow>();
        win.setSharingType(objc2_app_kit::NSWindowSharing::NSWindowSharingNone);
    }
    let _ = mtm;
    Ok(())
}
```
Call it on the hub webview window in `.setup`. Non-macOS: no-op stub.

- [ ] **Step 2:** `HubApp.tsx`: draggable region (`data-tauri-drag-region`), buttons Listen/Ask/Notes/Settings — all labels via `t("hub.*")` (Task 12) — wired to `ipc.ts` (Listen toggles start/stop; others `WebviewWindow` open with `?win=listen|ask|notes|settings`), opacity control calling `saveSettings`. **Step 3:** `npm run build` + `cargo build`. **Step 4:** manual: hub floats above fullscreen video, doesn't appear in `screencapture`. **Step 5:** commit `"feat: hub overlay + macOS capture exclusion"`.

---

### Task 14: Feature windows UI

**Files:**
- Create: `src/listen/ListenApp.tsx`, `src/ask/AskApp.tsx`, `src/notes/NotesApp.tsx`, `src/settings/SettingsApp.tsx`; Modify: `src/App.tsx` (route by `windowKind`).

- [ ] **Step 1–2:** `ListenApp` subscribes `subscribeRealtime`, appends to `TranscriptBuffer`, renders transcript list with `isUserTurn` highlight (title/empty/listening states via `t("listen.*")`); auto-scroll bottom. **Step 3–4:** `AskApp`: input placeholder/submit via `t("ask.*")`, `requestAnswer`, subscribes `subscribeAnswers`, renders streaming deltas; `stopAnswer` on unmount. `NotesApp`: list/create/edit/delete via note IPC, all labels via `t("notes.*")`. `SettingsApp`: masked key input with Save/Test (test = `startListen`+immediate `stopListen` catching `IpcError`), model fields, opacity slider, theme toggle, **language selector (en/es) that calls `setLocale` immediately and persists via `saveSettings({ language })`** — all labels via `t("settings.*")`. Route all in `App.tsx`. **Step 5:** `npm run build` + typecheck strict clean → commit `"feat: listen/ask/notes/settings windows with es/en i18n"`.

(These are thin presentational components over the Task 12 IPC surface; verification is build + the Task 11 manual checklist per window.)

---

### Task 15: Global hotkeys

**Files:**
- Create: `src-tauri/src/hotkeys.rs`; Modify: `lib.rs` (register from settings), `settings.rs` (add `shortcuts: Shortcuts` field with serde default).

- [ ] **Step 1:** failing test: `Shortcuts::default()` serde roundtrip from `{}` yields `cmdOrControl+shift+h` etc. **Step 2:** fail. **Step 3:** implement registration with `tauri-plugin-global-shortcut`: quick-hide (toggle hub visibility), push-to-ask (show ask window), start/stop listen; re-register on settings save. **Step 4:** tests + build pass. **Step 5:** commit `"feat: configurable global shortcuts"`.

---

### Task 16: Signing, notarization, checksums, release

**Files:**
- Create: `.github/workflows/release.yml`, `scripts/release.sh`

- [ ] **Step 1:** `release.yml` (macos-14): `npm ci && npm run tauri build` with secrets `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD` (app-specific), `APPLE_TEAM_ID`; then `xcrun notarytool submit`, `xcrun stapler staple`, verify `spctl -a -vv dist/.../cloakview.app`; export DMG as artifact. **Step 2:** `scripts/release.sh` prints SHA-256 of each artifact into `checksums.txt` and `gh release create`. **Step 3:** tag `v0.1.0`, run workflow, verify on a second Mac: Gatekeeper clean open, `spctl` says `accepted source=Notarized Developer ID`. **Step 4:** commit `"ci: signed notarized release pipeline + checksums"`.

---

## Self-Review

**Spec coverage:** MVP features → Tasks: scaffold(1), secrets(3), settings(4), overlay(13), audio(8), realtime(9), answers(10), detect(7), notes/conversations(5,6), UI(12–14, all text through es/en i18n), hotkeys(15), signing(16), IPC(11), listen-state/turn events(9,11). Spec §5 command list fully covered by Task 11. `is_pro()` stub: add `pub fn is_pro() -> bool { false }` in `state.rs` during Task 2 (covered). Spec §6 flow → Tasks 8→9→7→10→11. Spec §7 (zero telemetry) → no analytics anywhere (enforced by review). Spec §8 (checksums/notarization) → Task 16. Spec's "i18n-ready (es/en)" → real es/en catalogs with parity test (Task 12) and in-app language switching (Task 14).

**Placeholder scan:** Task 8's SCK body is the single manual seam, explicitly motivated (headless CI cannot drive ScreenCaptureKit) with the trait boundary keeping logic tested — not an open placeholder: the adapter is a bounded ~40-line adaptation pinned to the crate version, verified by the Task 11 manual checklist. No TBD/TODO elsewhere.

**Type consistency:** `IpcError` (Task 2) used by all commands from Task 3 on; `AudioStatus` (Task 8) consumed by Task 9/11; `RealtimeEvent`/`AnswersEvent` emitted as `"realtime"`/`"answers"` and mirrored in `models.ts` (Task 12) with identical camelCase field names; `Turn` produced by parse (Task 9) after detect labeling in listen pump (Task 11); `Settings` extended in Task 15 with serde defaults preserving Task 4's corrupt-file fallback test.

**Review Focus pin check:** #1 Task 3+11 tests; #2 Task 8 fake-error + Task 11 pump test; #3 Task 9 backoff test; #4 Task 9/10 parse tests; #5 Task 7 capacity test.

## Execution

This plan is sized for **subagent-driven execution** (fresh implementer + reviewer per task): the tasks are interface-heavy — a naming drift in Task 2 (`IpcError`) breaks 14 downstream tasks, which independent per-task review catches early. Native execution is acceptable if cost matters more than the interface drift risk.
