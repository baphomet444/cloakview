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
  featureWindowPosition,
  hideAll,
  isListening,
  openOrFocus,
  type HideApi,
  type WindowApi,
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
  const AT = { x: 120, y: 340 };

  it.each(FEATURE_KINDS)("%s: an overlay window — undecorated, transparent, always on top, off the taskbar, at /?win=<kind>", (kind) => {
    const { width, height } = FEATURE_WINDOWS[kind];
    expect(featureWindowOptions(kind, "Title", AT)).toMatchObject({
      url: `/?win=${kind}`,
      title: "Title",
      width,
      height,
      decorations: false,
      transparent: true,
      alwaysOnTop: true,
      skipTaskbar: true,
      x: AT.x,
      y: AT.y,
    });
  });

  // The product's whole point. These windows carry the transcript and the suggested
  // replies, and only the hub is excluded from capture at startup (`windows_ext`), so
  // each window the hub creates must ask for it itself. On macOS this sets
  // `NSWindow.sharingType = .none`, the same call `make_overlay` makes.
  it.each(FEATURE_KINDS)("%s: is excluded from screenshots and screen sharing (contentProtected)", (kind) => {
    expect(featureWindowOptions(kind, "Title", AT).contentProtected).toBe(true);
  });

  // Without it the window vanishes when another app goes full screen.
  it.each(FEATURE_KINDS)("%s: stays visible over full-screen apps (visibleOnAllWorkspaces)", (kind) => {
    expect(featureWindowOptions(kind, "Title", AT).visibleOnAllWorkspaces).toBe(true);
  });

  // Shown only by `promote`, once it is an overlay panel, so it never flashes as an
  // ordinary (capturable, Space-bound) window first.
  it.each(FEATURE_KINDS)("%s: is created hidden (visible: false)", (kind) => {
    expect(featureWindowOptions(kind, "Title", AT).visible).toBe(false);
  });
});

describe("featureWindowPosition", () => {
  const screen = { width: 1710, height: 1112 };
  const size = { width: 420, height: 360 };

  it("opens just below the hub, aligned with its left edge, when it fits", () => {
    const hub = { x: 645, y: 40, width: 420, height: 58 };
    expect(featureWindowPosition(hub, size, screen)).toEqual({ x: 645, y: 40 + 58 + 8 });
  });

  it("fits exactly at the bottom edge and still goes below", () => {
    const hub = { x: 0, y: screen.height - size.height - 8 - 58, width: 420, height: 58 };
    expect(featureWindowPosition(hub, size, screen).y).toBe(screen.height - size.height);
  });

  it("jumps above the hub when there is no room below", () => {
    const hub = { x: 645, y: 900, width: 420, height: 58 };
    expect(featureWindowPosition(hub, size, screen)).toEqual({ x: 645, y: 900 - 360 - 8 });
  });

  it("is pulled left so it does not run past the right edge", () => {
    const hub = { x: 1500, y: 40, width: 420, height: 58 };
    expect(featureWindowPosition(hub, size, screen).x).toBe(1710 - 420);
  });

  it("never starts left of the screen (a hub dragged partly off the left edge)", () => {
    const hub = { x: -60, y: 40, width: 420, height: 58 };
    expect(featureWindowPosition(hub, size, screen).x).toBe(0);
  });

  it("starts at 0 when it is wider than the screen", () => {
    const hub = { x: 300, y: 40, width: 420, height: 58 };
    expect(featureWindowPosition(hub, { width: 2000, height: 360 }, screen).x).toBe(0);
  });

  it("never goes above the top edge when it fits neither below nor above", () => {
    const hub = { x: 0, y: 100, width: 420, height: 58 };
    expect(featureWindowPosition(hub, { width: 420, height: 1100 }, screen).y).toBe(0);
  });

  it("never overlaps the hub when it fits below or above", () => {
    for (const hubY of [0, 200, 500, 800, 1054]) {
      const hub = { x: 645, y: hubY, width: 420, height: 58 };
      const { y } = featureWindowPosition(hub, size, screen);
      const overlaps = y < hub.y + hub.height && y + size.height > hub.y;
      expect(overlaps, `hub at y=${hubY}`).toBe(false);
    }
  });
});

describe("openOrFocus", () => {
  function fake(exists: boolean, fail?: "create" | "promote") {
    const calls: string[] = [];
    const api: WindowApi = {
      exists: (label) => {
        calls.push(`exists:${label}`);
        return Promise.resolve(exists);
      },
      create: (kind) => {
        calls.push(`create:${kind}`);
        return fail === "create" ? Promise.reject(new Error("denied")) : Promise.resolve();
      },
      promote: (kind) => {
        calls.push(`promote:${kind}`);
        return fail === "promote" ? Promise.reject(new Error("no panel")) : Promise.resolve();
      },
    };
    return { api, calls };
  }

  it("creates the window when none exists, then promotes it (which is what shows it)", async () => {
    const { api, calls } = fake(false);
    await openOrFocus("notes", api);
    expect(calls).toEqual(["exists:notes", "create:notes", "promote:notes"]);
  });

  it("promotes an existing window again instead of creating a duplicate", async () => {
    const { api, calls } = fake(true);
    await openOrFocus("ask", api);
    // Looked up by the kind as label, never created; `Close` only hid it, and promote
    // shows it as a panel (no Tauri show/setFocus, which would activate the app).
    expect(calls).toEqual(["exists:ask", "promote:ask"]);
  });

  it("lets a failure to create propagate, and never promotes a window that does not exist", async () => {
    const { api, calls } = fake(false, "create");
    await expect(openOrFocus("settings", api)).rejects.toThrow("denied");
    expect(calls).toEqual(["exists:settings", "create:settings"]);
  });

  it("lets a failure to promote propagate, so the hub shows an error instead of failing silently", async () => {
    const { api } = fake(true, "promote");
    await expect(openOrFocus("ask", api)).rejects.toThrow("no panel");
  });
});

describe("hideAll", () => {
  function fake(failing: readonly string[] = []) {
    const calls: string[] = [];
    const api: HideApi = {
      hideFeature: (kind) => {
        calls.push(`hide:${kind}`);
        return failing.includes(kind) ? Promise.reject(new Error(`${kind} gone`)) : Promise.resolve();
      },
      hideHub: () => {
        calls.push("hide:hub");
        return failing.includes("hub") ? Promise.reject(new Error("hub stuck")) : Promise.resolve();
      },
    };
    return { api, calls };
  }

  it("hides every feature window, then the hub last", async () => {
    const { api, calls } = fake();
    await hideAll(api);
    expect(calls.slice(0, -1).sort()).toEqual(FEATURE_KINDS.map((k) => `hide:${k}`).sort());
    expect(calls.at(-1)).toBe("hide:hub");
    expect(calls).toHaveLength(FEATURE_KINDS.length + 1);
  });

  it("keeps going when one feature window fails, and still hides the hub", async () => {
    const { api, calls } = fake(["ask"]);
    await expect(hideAll(api)).resolves.toBeUndefined();
    expect(calls).toContain("hide:notes");
    expect(calls).toContain("hide:settings");
    expect(calls.at(-1)).toBe("hide:hub");
  });

  it("lets a failure to hide the hub propagate, so the hub can show an error", async () => {
    const { api } = fake(["hub"]);
    await expect(hideAll(api)).rejects.toThrow("hub stuck");
  });
});
