/**
 * TypeScript mirror of what Rust serializes across the IPC boundary.
 *
 * Every name below is what serde *emits*, confirmed against the struct or enum
 * in `src-tauri/src` (the source file is noted on each type), not the Rust
 * field name:
 *   - DTO structs use `#[serde(rename_all = "camelCase")]`: `is_user_turn` is `isUserTurn`.
 *   - `i64` / `f64` are plain `number`; `Option<T>` is `T | null` when Rust sends it
 *     and an optional key when Rust receives it.
 *   - `ListenState` and `MicStatus` are camelCase, but `IpcError` has no `rename_all`,
 *     so its `kind` values are PascalCase. Different enums, different casing: don't share a helper.
 *
 * `RealtimeEvent` and `AnswersEvent` from the plan are deliberately absent: on the
 * Rust side they are internal `mpsc` transport only (`listen::pump` and
 * `commands::request_answer` map them onto the six spec events), so the frontend
 * never sees those shapes. What it receives is typed in `ipc.ts` (`EventPayloads`).
 */

/** One transcript line. `src-tauri/src/transcript.rs` (`Turn`, serialize only). */
export type Turn = {
  speaker: string;
  text: string;
  isUserTurn: boolean;
  /** Milliseconds since the epoch (Rust `i64`). */
  ts: number;
};

/** `src-tauri/src/db/notes.rs` (`Note`). `update_note` takes a whole `Note` back. */
export type Note = {
  id: string;
  title: string;
  content: string;
  tags: string[];
  favorite: boolean;
  archived: boolean;
  folderId: string | null;
  createdAt: number;
  updatedAt: number;
};

/** Input of `create_note`. `src-tauri/src/db/notes.rs` (`NewNote`): `tags` defaults to `[]`, `folderId` to none. */
export type NewNote = {
  title: string;
  content: string;
  tags?: string[];
  folderId?: string | null;
};

/** Input of `list_notes`. `src-tauri/src/db/notes.rs` (`NoteFilter`): every key optional, `archived` defaults to `false`. */
export type NoteFilter = {
  folderId?: string | null;
  archived?: boolean;
};

/** `src-tauri/src/db/conversations.rs` (`Conversation`). */
export type Conversation = {
  id: string;
  title: string;
  createdAt: number;
  updatedAt: number;
};

/** `src-tauri/src/db/conversations.rs` (`ConversationMessage`). `role` is free-form on the Rust side. */
export type ConversationMessage = {
  id: string;
  conversationId: string;
  role: string;
  content: string;
  model: string | null;
  createdAt: number;
};

/** Input of `add_message`. `src-tauri/src/db/conversations.rs` (`NewMessage`): `model` may be left out. */
export type NewMessage = {
  conversationId: string;
  role: string;
  content: string;
  model?: string | null;
};

/**
 * Global shortcut accelerators. `src-tauri/src/hotkeys.rs` (`Shortcuts`), e.g.
 * `cmdOrControl+shift+h`. A blank string turns that action off.
 */
export type Shortcuts = {
  quickHide: string;
  pushToAsk: string;
  toggleListen: string;
  readScreen: string;
};

/** `src-tauri/src/license.rs` (`License`): a `status` tag with the key as content. */
export type License =
  | { status: "inactive" }
  | { status: "pendingValidation"; key: string }
  | { status: "active"; key: string };

/** `src-tauri/src/state.rs` (`Settings`). `theme` and `language` are free-form strings in Rust. */
export type Settings = {
  language: string;
  theme: string;
  opacity: number;
  answersModel: string;
  realtimeModel: string;
  /** Matched against each transcript line to decide `turn-detected`. Seeded by Rust on a clean install. */
  userKeywords: string[];
  userName: string | null;
  /** Re-registered by Rust whenever a save changes them. */
  shortcuts: Shortcuts;
  /** Also transcribe the user's microphone (read when listening starts). */
  includeMic: boolean;
};

/**
 * Input of `save_settings`. `src-tauri/src/settings.rs` (`SettingsPatch`): omit a key to leave it unchanged.
 * `userName` is a `string`, not `string | null`: Rust can't tell `null` from "absent", so
 * to *clear* the name send a blank string (`""`).
 */
export type SettingsPatch = {
  language?: string;
  theme?: string;
  opacity?: number;
  answersModel?: string;
  realtimeModel?: string;
  userKeywords?: string[];
  userName?: string;
  /** All three at once: it replaces the saved set. */
  shortcuts?: Shortcuts;
  includeMic?: boolean;
};

/** Microphone permission. `src-tauri/src/platform.rs` (`MicStatus`): a plain camelCase string. */
export const MIC_STATUSES = ["notDetermined", "authorized", "denied", "restricted"] as const;
export type MicStatus = (typeof MIC_STATUSES)[number];

/** `finish_session`. `src-tauri/src/commands.rs` (`SessionSummary`): what the interview that just ended held. */
export type SessionSummary = {
  /** Lines heard on the call. */
  heard: number;
  /** Suggested answers given. */
  suggestions: number;
  /** Whether the suggested answers were saved as a note. */
  noteSaved: boolean;
};

/** `get_app_info`. `src-tauri/src/commands.rs` (`AppInfo`). */
export type AppInfo = {
  name: string;
  version: string;
};

/**
 * Payload of the `listen-state` event. `src-tauri/src/listen.rs` (`ListenState`):
 * internally tagged on `kind`, camelCase — `{ kind: "listening" }`, `{ kind: "error", message }`.
 */
export const LISTEN_STATE_KINDS = ["listening", "connected", "reconnecting", "stopped", "error"] as const;
export type ListenState =
  | { kind: Exclude<(typeof LISTEN_STATE_KINDS)[number], "error"> }
  | { kind: "error"; message: string };

/**
 * What a failed command rejects with. `src-tauri/src/error.rs` (`IpcError`): adjacently
 * tagged (`kind` + `message`) with **no** `rename_all`, so `kind` is the PascalCase variant
 * name. Unit variants send no `message` at all.
 */
export const IPC_ERROR_UNIT_KINDS = ["MissingApiKey", "PermissionDenied", "MicPermissionDenied", "ProRequired"] as const;
export const IPC_ERROR_MESSAGE_KINDS = [
  "Keychain",
  "Db",
  "Audio",
  "Network",
  "Serialize",
  "InvalidUrl",
  "OpenFailed",
  "License",
] as const;
export type IpcError =
  | { kind: (typeof IPC_ERROR_UNIT_KINDS)[number] }
  | { kind: (typeof IPC_ERROR_MESSAGE_KINDS)[number]; message: string };
