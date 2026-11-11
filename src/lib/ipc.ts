/**
 * Typed wrappers over Tauri's `invoke`/`listen`: the only place the frontend
 * touches the Rust IPC surface.
 *
 * Nothing here is stringly typed. `CommandSignatures` maps each command name to
 * its argument and result types, and `call` only accepts a name from that map,
 * so a typo, a wrong argument or a wrong result type is a compile error. The
 * names are exactly the ones Rust registers in `ipc_commands![...]`
 * (`src-tauri/src/lib.rs`, design.md §5); `ipc.test.ts` reads that file and
 * fails if `COMMANDS` and Rust ever differ. Events work the same way against
 * `src-tauri/src/events.rs`.
 *
 * Two conventions worth knowing:
 *  - Tauri turns a Rust `snake_case` argument into `camelCase` on the JS side,
 *    so Rust's `conversation_id: String` is `{ conversationId }` here.
 *  - A failed command rejects with the `IpcError` Rust serialized, unchanged;
 *    use `isIpcError` to narrow a caught value.
 */
import { invoke, type InvokeArgs } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  IPC_ERROR_MESSAGE_KINDS,
  IPC_ERROR_UNIT_KINDS,
  type AppInfo,
  type Conversation,
  type ConversationMessage,
  type IpcError,
  type ListenState,
  type MicStatus,
  type NewMessage,
  type NewNote,
  type Note,
  type NoteFilter,
  type Settings,
  type SettingsPatch,
  type Turn,
} from "./models";

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/** Argument and result types of every Rust command. Keys are the exact registered names. */
export interface CommandSignatures {
  get_api_key_status: { args: undefined; result: boolean };
  save_api_key: { args: { key: string }; result: void };
  clear_api_key: { args: undefined; result: void };
  get_settings: { args: undefined; result: Settings };
  save_settings: { args: { patch: SettingsPatch }; result: Settings };
  create_note: { args: { input: NewNote }; result: Note };
  list_notes: { args: { filter?: NoteFilter }; result: Note[] };
  update_note: { args: { note: Note }; result: Note };
  delete_note: { args: { id: string }; result: void };
  create_conversation: { args: { title: string }; result: Conversation };
  add_message: { args: { input: NewMessage }; result: ConversationMessage };
  list_messages: { args: { conversationId: string }; result: ConversationMessage[] };
  delete_conversation: { args: { id: string }; result: void };
  start_listen: { args: undefined; result: void };
  stop_listen: { args: undefined; result: void };
  request_answer: { args: { question?: string }; result: void };
  stop_answer: { args: undefined; result: void };
  get_mic_status: { args: undefined; result: MicStatus };
  get_app_info: { args: undefined; result: AppInfo };
  open_external: { args: { url: string }; result: void };
  promote_overlay: { args: { label: string }; result: void };
}

/**
 * The runtime list of the same names, for the contract test to compare with Rust's
 * `ipc_commands![...]`. `satisfies` makes any name not in `CommandSignatures` an
 * error here; the test checks the reverse (no command left out).
 */
export const COMMANDS = [
  "get_api_key_status",
  "save_api_key",
  "clear_api_key",
  "get_settings",
  "save_settings",
  "create_note",
  "list_notes",
  "update_note",
  "delete_note",
  "create_conversation",
  "add_message",
  "list_messages",
  "delete_conversation",
  "start_listen",
  "stop_listen",
  "request_answer",
  "stop_answer",
  "get_mic_status",
  "get_app_info",
  "open_external",
  "promote_overlay",
] as const satisfies readonly (keyof CommandSignatures)[];

export type CommandName = (typeof COMMANDS)[number];

type ArgsOf<C extends CommandName> = CommandSignatures[C]["args"];
type ResultOf<C extends CommandName> = CommandSignatures[C]["result"];
/** No argument object for commands that take none; exactly the declared one otherwise. */
type CallArgs<C extends CommandName> = ArgsOf<C> extends undefined ? [] : [args: ArgsOf<C>];

function call<C extends CommandName>(command: C, ...args: CallArgs<C>): Promise<ResultOf<C>> {
  // `CallArgs<C>` is `[]` or a one-element tuple; the cast is only to read that element generically.
  const [payload] = args as unknown as [InvokeArgs?];
  return payload === undefined ? invoke<ResultOf<C>>(command) : invoke<ResultOf<C>>(command, payload);
}

// secrets
export function saveApiKey(key: string): Promise<void> {
  return call("save_api_key", { key });
}
/** Whether a key is stored. Invokes `get_api_key_status` (the spec's name); resolves a plain boolean. */
export function hasApiKey(): Promise<boolean> {
  return call("get_api_key_status");
}
export function clearApiKey(): Promise<void> {
  return call("clear_api_key");
}

// settings
export function getSettings(): Promise<Settings> {
  return call("get_settings");
}
export function saveSettings(patch: SettingsPatch): Promise<Settings> {
  return call("save_settings", { patch });
}

// notes
export function createNote(input: NewNote): Promise<Note> {
  return call("create_note", { input });
}
export function listNotes(filter?: NoteFilter): Promise<Note[]> {
  return call("list_notes", { filter });
}
export function updateNote(note: Note): Promise<Note> {
  return call("update_note", { note });
}
export function deleteNote(id: string): Promise<void> {
  return call("delete_note", { id });
}

// conversations
export function createConversation(title: string): Promise<Conversation> {
  return call("create_conversation", { title });
}
export function addMessage(input: NewMessage): Promise<ConversationMessage> {
  return call("add_message", { input });
}
export function listMessages(conversationId: string): Promise<ConversationMessage[]> {
  return call("list_messages", { conversationId });
}
export function deleteConversation(id: string): Promise<void> {
  return call("delete_conversation", { id });
}

// listen / answer
/** Rejects with `MissingApiKey` (no key stored) or `PermissionDenied` (screen recording off). */
export function startListen(): Promise<void> {
  return call("start_listen");
}
export function stopListen(): Promise<void> {
  return call("stop_listen");
}
/** The answer streams back over the `answer-delta` / `answer-done` / `error` events, not this promise. */
export function requestAnswer(question?: string): Promise<void> {
  return call("request_answer", { question });
}
export function stopAnswer(): Promise<void> {
  return call("stop_answer");
}

// platform
export function getMicStatus(): Promise<MicStatus> {
  return call("get_mic_status");
}
export function getAppInfo(): Promise<AppInfo> {
  return call("get_app_info");
}
/** Rust accepts only `http`/`https` and rejects anything else with `InvalidUrl`. */
export function openExternal(url: string): Promise<void> {
  return call("open_external", { url });
}

/**
 * Turns the window `label` into an overlay panel (excluded from capture, over
 * full-screen Spaces, non-activating) and shows it. Safe to call again.
 */
export function promoteOverlay(label: string): Promise<void> {
  return call("promote_overlay", { label });
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/** Narrows a caught value to the `IpcError` Rust serialized. A message-carrying kind must carry a string `message`. */
export function isIpcError(value: unknown): value is IpcError {
  if (typeof value !== "object" || value === null) return false;
  const { kind, message } = value as { kind?: unknown; message?: unknown };
  if (typeof kind !== "string") return false;
  if ((IPC_ERROR_UNIT_KINDS as readonly string[]).includes(kind)) return true;
  return (IPC_ERROR_MESSAGE_KINDS as readonly string[]).includes(kind) && typeof message === "string";
}

// ---------------------------------------------------------------------------
// Events (design.md §5, "Eventos Rust → frontend")
// ---------------------------------------------------------------------------

/**
 * One Tauri event per thing that happens, each with a bare payload: register one
 * listener per event and never switch on a `kind` inside it (the one exception,
 * `listen-state`, is genuinely one event with several states).
 */
export interface EventPayloads {
  "transcript-delta": Turn;
  "turn-detected": null;
  "answer-delta": string;
  "answer-done": null;
  "listen-state": ListenState;
  error: string;
  /** The push-to-ask global shortcut, sent to the hub only. No data. */
  "push-to-ask": null;
}

/** Runtime list of the same names, compared by the test with `src-tauri/src/events.rs`. */
export const EVENTS = [
  "transcript-delta",
  "turn-detected",
  "answer-delta",
  "answer-done",
  "listen-state",
  "error",
  "push-to-ask",
] as const satisfies readonly (keyof EventPayloads)[];

export type EventName = (typeof EVENTS)[number];

function subscribe<E extends EventName>(event: E, handler: (payload: EventPayloads[E]) => void): Promise<UnlistenFn> {
  return listen<EventPayloads[E]>(event, (envelope) => handler(envelope.payload));
}

/** Each resolves to Tauri's `unlisten` function: call it to stop listening. */
export function subscribeTranscriptDelta(handler: (turn: Turn) => void): Promise<UnlistenFn> {
  return subscribe("transcript-delta", handler);
}
/** Fires when it is the user's turn to speak (the overlay's cue to suggest an answer). No data. */
export function subscribeTurnDetected(handler: () => void): Promise<UnlistenFn> {
  return subscribe("turn-detected", () => handler());
}
export function subscribeAnswerDelta(handler: (text: string) => void): Promise<UnlistenFn> {
  return subscribe("answer-delta", handler);
}
export function subscribeAnswerDone(handler: () => void): Promise<UnlistenFn> {
  return subscribe("answer-done", () => handler());
}
export function subscribeListenState(handler: (state: ListenState) => void): Promise<UnlistenFn> {
  return subscribe("listen-state", handler);
}
/** Shared by realtime and answer failures: a plain message string. */
export function subscribeError(handler: (message: string) => void): Promise<UnlistenFn> {
  return subscribe("error", handler);
}
/** Fires when the push-to-ask global shortcut is pressed: the hub opens the Ask window. No data. */
export function subscribePushToAsk(handler: () => void): Promise<UnlistenFn> {
  return subscribe("push-to-ask", () => handler());
}
