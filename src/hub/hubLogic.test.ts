import { describe, expect, it, vi } from "vitest";
import { en } from "../lib/locales/en";
import { es } from "../lib/locales/es";
import {
  IPC_ERROR_MESSAGE_KINDS,
  IPC_ERROR_UNIT_KINDS,
  type IpcError,
  type ListenState,
} from "../lib/models";
import {
  FEATURE_KINDS,
  FEATURE_WINDOWS,
  errorCopyKey,
  featureWindowOptions,
  isListening,
  openOrFocus,
  type WindowApi,
  type WindowHandle,
} from "./hubLogic";

// hubLogic reaches ipc.ts for `isIpcError`; keep Tauri's runtime out of these node tests.
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));

describe("isListening", () => {
  it.each<[string, ListenState | null, boolean]>([
    ["no state yet", null, false],
    ["listening", { kind: "listening" }, true],
    ["connected", { kind: "connected" }, true],
    ["reconnecting (the session is still up)", { kind: "reconnecting" }, true],
    ["stopped", { kind: "stopped" }, false],
    ["error", { kind: "error", message: "boom" }, false],
  ])("%s -> %s", (_label, state, expected) => {
    expect(isListening(state)).toBe(expected);
  });
});

describe("errorCopyKey", () => {
  it.each<[IpcError, string]>([
    [{ kind: "MissingApiKey" }, "errors.missingApiKey"],
    [{ kind: "PermissionDenied" }, "errors.permissionDenied"],
    [{ kind: "InvalidUrl", message: "x" }, "errors.invalidUrl"],
    [{ kind: "OpenFailed", message: "x" }, "errors.openFailed"],
  ])("%j -> %s", (error, key) => {
    expect(errorCopyKey(error)).toBe(key);
  });

  it("falls back to the generic copy for kinds the UI does not explain, and for anything that is not an IpcError", () => {
    for (const kind of ["Keychain", "Db", "Audio", "Network", "Serialize"] as const) {
      expect(errorCopyKey({ kind, message: "x" })).toBe("errors.generic");
    }
    for (const value of [new Error("boom"), "boom", null, undefined, { kind: "listening" }, {}]) {
      expect(errorCopyKey(value)).toBe("errors.generic");
    }
  });

  it("only ever returns a key that exists in both catalogs, for every IpcError kind Rust can send", () => {
    const everyError: IpcError[] = [
      ...IPC_ERROR_UNIT_KINDS.map((kind): IpcError => ({ kind })),
      ...IPC_ERROR_MESSAGE_KINDS.map((kind): IpcError => ({ kind, message: "m" })),
    ];
    for (const error of everyError) {
      const key = errorCopyKey(error);
      expect(en, `en lacks ${key}`).toHaveProperty([key]);
      expect(es, `es lacks ${key}`).toHaveProperty([key]);
    }
  });
});

describe("feature windows", () => {
  it("the hub opens exactly ask, notes and settings, and each label is its kind", () => {
    expect([...FEATURE_KINDS]).toEqual(["ask", "notes", "settings"]);
    expect(Object.keys(FEATURE_WINDOWS).sort()).toEqual([...FEATURE_KINDS].sort());
  });

  it("each window's title comes from the catalogs, and its size is usable", () => {
    for (const kind of FEATURE_KINDS) {
      const { titleKey, width, height } = FEATURE_WINDOWS[kind];
      expect(en, `en lacks ${titleKey}`).toHaveProperty([titleKey]);
      expect(es, `es lacks ${titleKey}`).toHaveProperty([titleKey]);
      expect(width).toBeGreaterThan(0);
      expect(height).toBeGreaterThan(0);
    }
  });
});

describe("featureWindowOptions", () => {
  it.each(FEATURE_KINDS)("%s: an overlay window — undecorated, transparent, always on top, off the taskbar, at /?win=<kind>", (kind) => {
    const { width, height } = FEATURE_WINDOWS[kind];
    expect(featureWindowOptions(kind, "Title")).toMatchObject({
      url: `/?win=${kind}`,
      title: "Title",
      width,
      height,
      decorations: false,
      transparent: true,
      alwaysOnTop: true,
      skipTaskbar: true,
    });
  });

  // The product's whole point. These windows carry the transcript and the suggested
  // replies, and only the hub is excluded from capture at startup (`windows_ext`), so
  // each window the hub creates must ask for it itself. On macOS this sets
  // `NSWindow.sharingType = .none`, the same call `exclude_from_capture` makes.
  it.each(FEATURE_KINDS)("%s: is excluded from screenshots and screen sharing (contentProtected)", (kind) => {
    expect(featureWindowOptions(kind, "Title").contentProtected).toBe(true);
  });
});

describe("openOrFocus", () => {
  function fake(existing: WindowHandle | null) {
    const calls: string[] = [];
    const api: WindowApi = {
      find: (label) => {
        calls.push(`find:${label}`);
        return Promise.resolve(existing);
      },
      create: (kind) => {
        calls.push(`create:${kind}`);
        return Promise.resolve();
      },
    };
    return { api, calls };
  }

  it("focuses a window that already exists instead of creating a duplicate, un-hiding it first", async () => {
    const calls: string[] = [];
    const handle: WindowHandle = {
      show: () => {
        calls.push("show");
        return Promise.resolve();
      },
      setFocus: () => {
        calls.push("setFocus");
        return Promise.resolve();
      },
    };
    const { api, calls: apiCalls } = fake(handle);

    await openOrFocus("ask", api);

    expect(apiCalls).toEqual(["find:ask"]); // looked it up by the kind as label, never created
    expect(calls).toEqual(["show", "setFocus"]); // `Close` only hides, so show before focus
  });

  it("creates the window when none exists", async () => {
    const { api, calls } = fake(null);
    await openOrFocus("notes", api);
    expect(calls).toEqual(["find:notes", "create:notes"]);
  });

  it("lets a failure to create propagate, so the hub can show an error instead of failing silently", async () => {
    const failing: WindowApi = {
      find: () => Promise.resolve(null),
      create: () => Promise.reject(new Error("denied")),
    };
    await expect(openOrFocus("settings", failing)).rejects.toThrow("denied");
  });
});
