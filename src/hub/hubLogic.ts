/**
 * The decisions the hub makes, kept free of React and of Tauri so they can be
 * unit-tested: which listen state counts as "on", which catalog copy explains a
 * failed command, and whether a feature window is opened or just focused.
 * `HubApp.tsx` supplies the real Tauri window API and renders the result.
 */
import type { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { isIpcError } from "../lib/ipc";
import type { CatalogKey } from "../lib/i18n";
import type { ListenState } from "../lib/models";

/**
 * Whether a listen session is running or on its way. `reconnecting` counts: the
 * session is still up and recovering, so the Listen button must still say "stop".
 */
export function isListening(state: ListenState | null): boolean {
  return state !== null && (state.kind === "listening" || state.kind === "connected" || state.kind === "reconnecting");
}

/**
 * The catalog key of the copy for a failed command. Only the errors the user can
 * act on get their own text; everything else — other Rust errors, or a rejection
 * that is not an `IpcError` at all (a thrown `Error`, a plugin failure) — is the generic copy.
 * Rust's raw message is never shown: it is English-only and not written for users.
 */
export function errorCopyKey(error: unknown): CatalogKey {
  if (!isIpcError(error)) return "errors.generic";
  switch (error.kind) {
    case "MissingApiKey":
      return "errors.missingApiKey";
    case "PermissionDenied":
      return "errors.permissionDenied";
    case "InvalidUrl":
      return "errors.invalidUrl";
    case "OpenFailed":
      return "errors.openFailed";
    default:
      return "errors.generic";
  }
}

/** The overlay windows the hub opens. The window label is the kind itself, and the URL is `/?win=<kind>`. */
export type FeatureKind = "ask" | "notes" | "settings";

export const FEATURE_KINDS: readonly FeatureKind[] = ["ask", "notes", "settings"];

/** Per-window title (a catalog key, never literal text) and starting size. */
export const FEATURE_WINDOWS: Record<FeatureKind, { titleKey: CatalogKey; width: number; height: number }> = {
  ask: { titleKey: "hub.ask", width: 420, height: 360 },
  notes: { titleKey: "hub.notes", width: 420, height: 420 },
  settings: { titleKey: "hub.settings", width: 460, height: 480 },
};

/** The options Tauri's `new WebviewWindow(label, options)` accepts. */
export type FeatureWindowOptions = NonNullable<ConstructorParameters<typeof WebviewWindow>[1]>;

/**
 * How every overlay window the hub creates is built: undecorated, transparent,
 * always on top, off the taskbar — and `contentProtected`, so it never shows up in
 * a screenshot, a recording or a screen share. That last one matters most: these
 * windows carry the transcript and the suggested replies, and only the hub itself
 * is excluded at startup (`windows_ext::exclude_from_capture`). On macOS
 * `contentProtected` is `NSWindow.sharingType = .none`, the same call.
 * `title` is passed in (already translated) because this module does not touch i18n state.
 */
export function featureWindowOptions(kind: FeatureKind, title: string): FeatureWindowOptions {
  const { width, height } = FEATURE_WINDOWS[kind];
  return {
    url: `/?win=${kind}`,
    title,
    width,
    height,
    decorations: false,
    transparent: true,
    alwaysOnTop: true,
    skipTaskbar: true,
    contentProtected: true,
  };
}

/** The slice of Tauri's `WebviewWindow` that `openOrFocus` needs. */
export interface WindowHandle {
  show(): Promise<void>;
  setFocus(): Promise<void>;
}

export interface WindowApi {
  /** The existing window with this label, or `null`. */
  find(label: string): Promise<WindowHandle | null>;
  /** Creates the window for `kind`; resolves once it exists, rejects if it could not be created. */
  create(kind: FeatureKind): Promise<void>;
}

/**
 * Brings the window for `kind` to the front, creating it only if there is none.
 * An existing window is shown first because the hub's `Close` merely hides it;
 * focusing a hidden window would otherwise do nothing visible.
 */
export async function openOrFocus(kind: FeatureKind, api: WindowApi): Promise<void> {
  const existing = await api.find(kind);
  if (existing) {
    await existing.show();
    await existing.setFocus();
    return;
  }
  await api.create(kind);
}
