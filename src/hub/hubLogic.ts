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
    case "MicPermissionDenied":
      return "errors.micPermissionDenied";
    case "InvalidUrl":
      return "errors.invalidUrl";
    case "OpenFailed":
      return "errors.openFailed";
    default:
      return "errors.generic";
  }
}

/** The overlay windows the hub opens. The window label is the kind itself, and the URL is `/?win=<kind>`. */
export type FeatureKind = "ask" | "listen" | "notes" | "settings";

export const FEATURE_KINDS: readonly FeatureKind[] = ["ask", "listen", "notes", "settings"];

/** Per-window title (a catalog key, never literal text) and starting size. */
export const FEATURE_WINDOWS: Record<FeatureKind, { titleKey: CatalogKey; width: number; height: number }> = {
  ask: { titleKey: "hub.ask", width: 420, height: 360 },
  listen: { titleKey: "listen.title", width: 380, height: 360 },
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
 * is excluded at startup (`windows_ext::make_overlay`). On macOS
 * `contentProtected` is `NSWindow.sharingType = .none`, the same call.
 * `visibleOnAllWorkspaces` keeps them over other apps' full-screen Spaces.
 * `title` is passed in (already translated) because this module does not touch i18n state.
 * `at` is where it opens, in logical pixels (see `featureWindowPosition`).
 */
export function featureWindowOptions(
  kind: FeatureKind,
  title: string,
  at: { x: number; y: number },
): FeatureWindowOptions {
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
    visibleOnAllWorkspaces: true,
    // Created hidden: it is shown by `promote` once it is an overlay panel, so it never
    // flashes as an ordinary window first.
    visible: false,
    x: at.x,
    y: at.y,
  };
}

/** Gap, in logical pixels, between the hub and a feature window. */
export const FEATURE_WINDOW_GAP = 8;

/**
 * Posición de la ventana de feature: alineada con el hub y justo debajo.
 *
 * Everything is in logical pixels, relative to the hub's monitor (origin 0,0); the
 * caller converts. Below the hub if it fits on the screen, above it otherwise; `x`
 * follows the hub's left edge, kept on screen. It never overlaps the hub, which is
 * the point: an overlapping panel would sit on top of it and take its clicks. `y` is
 * also kept from going above the top edge when it fits neither below nor above.
 */
export function featureWindowPosition(
  hub: { x: number; y: number; width: number; height: number },
  size: { width: number; height: number },
  screen: { width: number; height: number },
): { x: number; y: number } {
  const below = hub.y + hub.height + FEATURE_WINDOW_GAP;
  const y = below + size.height <= screen.height ? below : Math.max(0, hub.y - size.height - FEATURE_WINDOW_GAP);
  const x = Math.max(0, Math.min(hub.x, screen.width - size.width));
  return { x, y };
}

/**
 * The rectangle a feature window is placed against. The live transcript sits beside
 * Ask instead of on top of it (both open when listening starts), so it is placed as
 * if the hub began where Ask ends; the others use the hub itself.
 */
export function anchorFor(
  kind: FeatureKind,
  hub: { x: number; y: number; width: number; height: number },
): { x: number; y: number; width: number; height: number } {
  if (kind !== "listen") return hub;
  return { ...hub, x: hub.x + FEATURE_WINDOWS.ask.width + FEATURE_WINDOW_GAP };
}

export interface WindowApi {
  /** Whether a window with this label already exists. */
  exists(label: string): Promise<boolean>;
  /** Creates the (hidden) window for `kind`; resolves once it exists, rejects if it could not be created. */
  create(kind: FeatureKind): Promise<void>;
  /** Makes the window for `kind` an overlay panel and shows it (`promoteOverlay`). */
  promote(kind: FeatureKind): Promise<void>;
}

/**
 * Brings the window for `kind` to the front, creating it only if there is none.
 * Either way it is shown only by `promote`, after it has become an overlay panel:
 * the window is created hidden, and an existing one (the hub's `Close` merely hides
 * it) is promoted again rather than shown with Tauri's `show`/`setFocus`.
 * `setFocus` in particular would activate the app, which is what pulls the user out
 * of another app's full-screen Space.
 */
export async function openOrFocus(kind: FeatureKind, api: WindowApi): Promise<void> {
  if (!(await api.exists(kind))) {
    await api.create(kind);
  }
  await api.promote(kind);
}

export interface HideApi {
  /** Hides the feature window `kind` if it exists; resolves when there is none. */
  hideFeature(kind: FeatureKind): Promise<void>;
  /** Hides the hub itself. */
  hideHub(): Promise<void>;
}

/**
 * `Close`: hides the whole overlay, not just the hub. Every feature window is hidden
 * first, each on its own: a failure on one (say, it closed in the meantime) is
 * ignored, so it cannot keep the rest on screen. The hub goes last, and a failure
 * there does propagate, because then the overlay is still visible and the hub should
 * say so. Like `openOrFocus`, it only talks to the api, never to the DOM or Tauri.
 */
export async function hideAll(api: HideApi): Promise<void> {
  await Promise.allSettled(FEATURE_KINDS.map((kind) => api.hideFeature(kind)));
  await api.hideHub();
}
