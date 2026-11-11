/**
 * The Rust <-> TypeScript contract for the IPC layer.
 *
 * `ipc.ts` and `models.ts` are a hand-written mirror of the Rust side, so this
 * file keeps the two honest. It reads the Rust sources as text (Vite `?raw`)
 * and compares: the command list against `ipc_commands![...]` in lib.rs, the
 * event names against events.rs, and the enum values (IpcError kinds,
 * MicStatus, ListenState) against their Rust definitions. A rename or an
 * omission on either side fails here, in CI, instead of in the running app.
 *
 * It then checks each wrapper against a mocked `invoke`/`listen`: right
 * command, right camelCase argument names, bare payloads handed to handlers.
 */
import { beforeEach, describe, expect, expectTypeOf, it, vi } from "vitest";
import errorRs from "../../src-tauri/src/error.rs?raw";
import eventsRs from "../../src-tauri/src/events.rs?raw";
import libRs from "../../src-tauri/src/lib.rs?raw";
import listenRs from "../../src-tauri/src/listen.rs?raw";
import platformRs from "../../src-tauri/src/platform.rs?raw";
import {
  IPC_ERROR_MESSAGE_KINDS,
  IPC_ERROR_UNIT_KINDS,
  LISTEN_STATE_KINDS,
  MIC_STATUSES,
  type ConversationMessage,
  type IpcError,
  type ListenState,
  type MicStatus,
  type Note,
  type Settings,
  type Turn,
} from "./models";

const invokeMock = vi.hoisted(() => vi.fn<(command: string, args?: Record<string, unknown>) => Promise<unknown>>());
const listenMock = vi.hoisted(() =>
  vi.fn<(event: string, handler: (event: { payload: unknown }) => void) => Promise<() => void>>(),
);
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/api/event", () => ({ listen: listenMock }));

import * as ipc from "./ipc";

// ---------------------------------------------------------------------------
// Reading the Rust side
// ---------------------------------------------------------------------------

const sorted = (values: readonly string[]): string[] => [...values].sort();
const lowerFirst = (s: string): string => s.charAt(0).toLowerCase() + s.slice(1);

/** The command names inside `ipc_commands![ ... ]`, the single place Rust registers them. */
function rustRegisteredCommands(): string[] {
  const body = /ipc_commands!\[([^\]]*)\]/.exec(libRs)?.[1];
  if (body === undefined) throw new Error("ipc_commands![...] not found in lib.rs");
  return body
    .split(",")
    .map((name) => name.trim())
    .filter((name) => name.length > 0);
}

/** The string values of every `pub const X: &str = "..."` in events.rs. */
function rustEventNames(): string[] {
  return [...eventsRs.matchAll(/pub const \w+: &str = "([^"]+)";/g)].map((m) => m[1]);
}

/**
 * Variants of `pub enum <name> { ... }`: `unit` ones (`A,`) and `withData` ones
 * (`A(String),` / `A { x: T },`). The closing brace is the first `}` in column 0.
 */
function rustEnum(source: string, name: string): { unit: string[]; withData: string[] } {
  const body = new RegExp(`pub enum ${name} \\{([\\s\\S]*?)\\n\\}`).exec(source)?.[1];
  if (body === undefined) throw new Error(`enum ${name} not found`);
  const unit: string[] = [];
  const withData: string[] = [];
  for (const raw of body.split("\n")) {
    const line = raw.trim();
    if (line === "" || line.startsWith("//") || line.startsWith("#[")) continue;
    const variant = /^(\w+)/.exec(line)?.[1];
    if (variant === undefined) continue;
    (/[({]/.test(line) ? withData : unit).push(variant);
  }
  return { unit, withData };
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

describe("command surface", () => {
  it("COMMANDS is exactly the set Rust registers in ipc_commands![...] (lib.rs)", () => {
    expect(sorted(ipc.COMMANDS)).toEqual(sorted(rustRegisteredCommands()));
    expect(new Set(ipc.COMMANDS).size).toBe(ipc.COMMANDS.length); // no duplicates
  });

  it("every command's signature entry and its COMMANDS entry agree (checked by tsc)", () => {
    expectTypeOf<ipc.CommandName>().toEqualTypeOf<keyof ipc.CommandSignatures>();
  });
});

type Case = readonly [command: ipc.CommandName, run: () => Promise<unknown>, args: Record<string, unknown> | undefined];

const note: Note = {
  id: "n1",
  title: "t",
  content: "c",
  tags: ["a"],
  favorite: false,
  archived: false,
  folderId: null,
  createdAt: 1,
  updatedAt: 2,
};

// One row per Rust command: the wrapper, and the exact camelCase arguments it must send.
const cases: Case[] = [
  ["get_api_key_status", () => ipc.hasApiKey(), undefined],
  ["save_api_key", () => ipc.saveApiKey("sk-1"), { key: "sk-1" }],
  ["clear_api_key", () => ipc.clearApiKey(), undefined],
  ["get_settings", () => ipc.getSettings(), undefined],
  ["save_settings", () => ipc.saveSettings({ opacity: 0.5 }), { patch: { opacity: 0.5 } }],
  ["create_note", () => ipc.createNote({ title: "t", content: "c" }), { input: { title: "t", content: "c" } }],
  ["list_notes", () => ipc.listNotes({ archived: true }), { filter: { archived: true } }],
  ["update_note", () => ipc.updateNote(note), { note }],
  ["delete_note", () => ipc.deleteNote("n1"), { id: "n1" }],
  ["create_conversation", () => ipc.createConversation("demo"), { title: "demo" }],
  [
    "add_message",
    () => ipc.addMessage({ conversationId: "c1", role: "user", content: "hi" }),
    { input: { conversationId: "c1", role: "user", content: "hi" } },
  ],
  // The camelCase trap: Rust's `conversation_id` argument is `conversationId` from JS.
  ["list_messages", () => ipc.listMessages("c1"), { conversationId: "c1" }],
  ["delete_conversation", () => ipc.deleteConversation("c1"), { id: "c1" }],
  ["start_listen", () => ipc.startListen(), undefined],
  ["stop_listen", () => ipc.stopListen(), undefined],
  ["request_answer", () => ipc.requestAnswer("¿qué opinas?"), { question: "¿qué opinas?" }],
  ["stop_answer", () => ipc.stopAnswer(), undefined],
  ["get_mic_status", () => ipc.getMicStatus(), undefined],
  ["get_app_info", () => ipc.getAppInfo(), undefined],
  ["open_external", () => ipc.openExternal("https://example.com"), { url: "https://example.com" }],
  ["promote_overlay", () => ipc.promoteOverlay("ask"), { label: "ask" }],
  ["finish_session", () => ipc.finishSession(), undefined],
  ["save_session_txt", () => ipc.saveSessionTxt(), undefined],
  ["read_screen", () => ipc.readScreen(), undefined],
];

describe("wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(null);
  });

  it.each(cases)("%s: the wrapper invokes exactly that command with the right arguments", async (command, run, args) => {
    await run();
    expect(invokeMock).toHaveBeenCalledTimes(1);
    if (args === undefined) {
      expect(invokeMock).toHaveBeenCalledWith(command);
    } else {
      expect(invokeMock).toHaveBeenCalledWith(command, args);
    }
  });

  it("has a wrapper test case for every command (none forgotten)", () => {
    expect(sorted(cases.map(([command]) => command))).toEqual(sorted(ipc.COMMANDS));
  });

  it("optional arguments may be left out", async () => {
    await ipc.listNotes();
    await ipc.requestAnswer();
    // `undefined` properties vanish in JSON, so Rust sees `None` for both.
    expect(invokeMock).toHaveBeenNthCalledWith(1, "list_notes", {});
    expect(invokeMock).toHaveBeenNthCalledWith(2, "request_answer", {});
  });

  it("resolves with what Rust returned", async () => {
    invokeMock.mockResolvedValueOnce(true);
    await expect(ipc.hasApiKey()).resolves.toBe(true);
    invokeMock.mockResolvedValueOnce("authorized");
    await expect(ipc.getMicStatus()).resolves.toBe("authorized");
  });

  it("rejects with the IpcError Rust serialized, untouched", async () => {
    const error: IpcError = { kind: "MissingApiKey" };
    invokeMock.mockRejectedValueOnce(error);
    await expect(ipc.startListen()).rejects.toBe(error);
  });

  it("argument and result types come from the models (checked by tsc)", () => {
    expectTypeOf(ipc.listMessages).parameters.toEqualTypeOf<[conversationId: string]>();
    expectTypeOf(ipc.listMessages).returns.resolves.toEqualTypeOf<ConversationMessage[]>();
    expectTypeOf(ipc.getSettings).returns.resolves.toEqualTypeOf<Settings>();
    expectTypeOf(ipc.getMicStatus).returns.resolves.toEqualTypeOf<MicStatus>();
    expectTypeOf(ipc.hasApiKey).returns.resolves.toEqualTypeOf<boolean>();
    expectTypeOf(ipc.updateNote).parameters.toEqualTypeOf<[note: Note]>();
  });
});

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

const turn: Turn = { speaker: "remote", text: "hola", isUserTurn: false, ts: 1 };
const listening: ListenState = { kind: "listening" };

describe("events", () => {
  const handlers = new Map<string, (event: { payload: unknown }) => void>();
  const unlisten = vi.fn();

  beforeEach(() => {
    handlers.clear();
    unlisten.mockReset();
    listenMock.mockReset();
    listenMock.mockImplementation((event, handler) => {
      handlers.set(event, handler);
      return Promise.resolve(unlisten);
    });
  });

  it("EVENTS is exactly the set of event names Rust emits (events.rs)", () => {
    expect(sorted(ipc.EVENTS)).toEqual(sorted(rustEventNames()));
    expectTypeOf<ipc.EventName>().toEqualTypeOf<keyof ipc.EventPayloads>();
  });

  // Each subscription: the spec event it listens on, the payload Rust sends, what the handler receives.
  const subscriptions: ReadonlyArray<
    readonly [event: ipc.EventName, subscribe: (handler: (payload: unknown) => void) => Promise<() => void>, payload: unknown]
  > = [
    ["transcript-delta", ipc.subscribeTranscriptDelta, turn],
    ["answer-delta", ipc.subscribeAnswerDelta, "Hel"],
    ["answer-started", ipc.subscribeAnswerStarted as (handler: (payload: unknown) => void) => Promise<() => void>, "screen"],
    ["listen-state", ipc.subscribeListenState, listening],
    ["error", ipc.subscribeError, "rate_limit"],
  ];

  it.each(subscriptions)("%s: listens on that event and hands the handler the bare payload", async (event, subscribe, payload) => {
    const received = vi.fn();
    const stop = await subscribe(received);

    expect(stop).toBe(unlisten); // the caller gets Tauri's unlisten function
    expect(listenMock).toHaveBeenCalledTimes(1);
    expect(listenMock.mock.calls[0][0]).toBe(event);

    handlers.get(event)?.({ payload });
    expect(received).toHaveBeenCalledTimes(1);
    expect(received).toHaveBeenCalledWith(payload); // the bare payload: not the envelope, no `kind` wrapper added
  });

  it.each([
    ["turn-detected", ipc.subscribeTurnDetected],
    ["answer-done", ipc.subscribeAnswerDone],
    ["push-to-ask", ipc.subscribePushToAsk],
  ] as const)("%s: carries no data, so the handler takes no arguments", async (event, subscribe) => {
    const received = vi.fn();
    await subscribe(received);
    expect(listenMock.mock.calls[0][0]).toBe(event);

    handlers.get(event)?.({ payload: null });
    expect(received).toHaveBeenCalledTimes(1);
    expect(received.mock.calls[0]).toEqual([]); // called with no arguments at all
  });

  it("has a subscribe wrapper for every event (none forgotten)", () => {
    const covered = [...subscriptions.map(([event]) => event), "turn-detected", "answer-done", "push-to-ask"];
    expect(sorted(covered)).toEqual(sorted(ipc.EVENTS));
  });
});

// ---------------------------------------------------------------------------
// Wire types that carry literal values
// ---------------------------------------------------------------------------

describe("wire types mirror the Rust enums", () => {
  it("IpcError kinds are Rust's variants as written (PascalCase), split into unit and message-carrying", () => {
    const { unit, withData } = rustEnum(errorRs, "IpcError");
    expect(sorted(IPC_ERROR_UNIT_KINDS)).toEqual(sorted(unit));
    expect(sorted(IPC_ERROR_MESSAGE_KINDS)).toEqual(sorted(withData));
    expectTypeOf<IpcError["kind"]>().toEqualTypeOf<
      (typeof IPC_ERROR_UNIT_KINDS)[number] | (typeof IPC_ERROR_MESSAGE_KINDS)[number]
    >();
  });

  it("MicStatus values are Rust's MicStatus variants in camelCase", () => {
    const { unit, withData } = rustEnum(platformRs, "MicStatus");
    expect(withData).toEqual([]);
    expect(sorted(MIC_STATUSES)).toEqual(sorted(unit.map(lowerFirst)));
    expectTypeOf<MicStatus>().toEqualTypeOf<(typeof MIC_STATUSES)[number]>();
  });

  it("ListenState kinds are Rust's ListenState variants in camelCase", () => {
    const { unit, withData } = rustEnum(listenRs, "ListenState");
    expect(sorted(LISTEN_STATE_KINDS)).toEqual(sorted([...unit, ...withData].map(lowerFirst)));
    expectTypeOf<ListenState["kind"]>().toEqualTypeOf<(typeof LISTEN_STATE_KINDS)[number]>();
  });

  it("the two error-ish enums use different casing, on purpose: IpcError is PascalCase, ListenState camelCase", () => {
    expect(IPC_ERROR_UNIT_KINDS).toContain("MissingApiKey");
    expect(LISTEN_STATE_KINDS).toContain("error");
  });
});

describe("isIpcError", () => {
  it.each<unknown>([
    { kind: "MissingApiKey" },
    { kind: "PermissionDenied" },
    { kind: "Db", message: "disk full" },
    { kind: "InvalidUrl", message: 'scheme "file" is not allowed' },
    { kind: "OpenFailed", message: "boom" },
  ])("accepts a real IpcError: %j", (value) => {
    expect(ipc.isIpcError(value)).toBe(true);
  });

  it.each<unknown>([
    null,
    undefined,
    "MissingApiKey",
    42,
    {},
    new Error("plain error"),
    { kind: "Nope" },
    { kind: "Db" }, // a message-carrying kind must carry its message
    { kind: "Db", message: 3 },
    { kind: "listening" }, // a ListenState is not an IpcError
  ])("rejects what is not an IpcError: %j", (value) => {
    expect(ipc.isIpcError(value)).toBe(false);
  });
});
