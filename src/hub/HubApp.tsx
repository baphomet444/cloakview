import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { currentMonitor, getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";
import { getLocale, localeFromSetting, setLocale, t, type CatalogKey, type Locale } from "../lib/i18n";
import {
  getSettings,
  promoteOverlay,
  saveSettings,
  startListen,
  stopListen,
  subscribeListenState,
  subscribePushToAsk,
} from "../lib/ipc";
import type { ListenState } from "../lib/models";
import {
  FEATURE_WINDOWS,
  errorCopyKey,
  featureWindowOptions,
  featureWindowPosition,
  hideAll,
  isListening,
  openOrFocus,
  type FeatureKind,
  type HideApi,
  type WindowApi,
} from "./hubLogic";

/** How long an error replaces the bar before it goes away on its own. */
const ERROR_DISMISS_MS = 6000;
/** Wait for the slider to settle before writing `settings.json`, instead of on every tick. */
const OPACITY_SAVE_DELAY_MS = 250;
/** Rust clamps to 0.1..=1.0 as well; below 0.1 the hub would be invisible and impossible to find. */
const MIN_OPACITY = 0.1;

/**
 * Where the feature window `kind` opens: below the hub (or above it, near the bottom
 * edge), so it never covers the hub and takes its clicks.
 *
 * Tauri reports the hub and the monitor in *physical* pixels, in global desktop
 * coordinates, while window options take *logical* pixels. So everything is turned
 * logical with the monitor's scale factor and made relative to the monitor before
 * `featureWindowPosition`, and the monitor's origin is added back after; otherwise a
 * Retina screen (scale 2) or a second monitor would put the window in the wrong place.
 */
async function positionBelowHub(kind: FeatureKind): Promise<{ x: number; y: number }> {
  const hub = getCurrentWindow();
  const [position, size, monitor] = await Promise.all([hub.outerPosition(), hub.outerSize(), currentMonitor()]);
  const scale = monitor?.scaleFactor ?? (await hub.scaleFactor());
  const origin = monitor ? monitor.position.toLogical(scale) : { x: 0, y: 0 };
  const screen = monitor ? monitor.size.toLogical(scale) : { width: Infinity, height: Infinity };
  const hubPos = position.toLogical(scale);
  const hubSize = size.toLogical(scale);
  const { width, height } = FEATURE_WINDOWS[kind];
  const at = featureWindowPosition(
    { x: hubPos.x - origin.x, y: hubPos.y - origin.y, width: hubSize.width, height: hubSize.height },
    { width, height },
    { width: screen.width, height: screen.height },
  );
  return { x: at.x + origin.x, y: at.y + origin.y };
}

/** The real Tauri windows behind `hideAll` (the `Close` button). */
const tauriHide: HideApi = {
  hideFeature: async (kind) => {
    const window = await WebviewWindow.getByLabel(kind);
    if (window) await window.hide();
  },
  hideHub: () => getCurrentWindow().hide(),
};

/** The real Tauri windows behind `openOrFocus`. */
const tauriWindows: WindowApi = {
  exists: async (label) => (await WebviewWindow.getByLabel(label)) !== null,
  // Created hidden (`visible: false`); resolves on `tauri://created`, so `promote`
  // only ever runs against a window Rust can already find by label.
  create: async (kind) => {
    const at = await positionBelowHub(kind);
    await new Promise<void>((resolve, reject) => {
      const created = new WebviewWindow(kind, featureWindowOptions(kind, t(FEATURE_WINDOWS[kind].titleKey), at));
      void created.once("tauri://created", () => resolve());
      void created.once("tauri://error", (event) => reject(new Error(String(event.payload))));
    });
  },
  promote: (kind) => promoteOverlay(kind),
};

/**
 * The floating control bar. The whole bar is a drag region (`data-tauri-drag-region`);
 * the buttons deliberately are not, or they could not be clicked. All text comes
 * from the i18n catalogs. A 420x58 window leaves no room for a status line, so an
 * error takes over the bar for a few seconds instead.
 */
export function HubApp() {
  const [locale, setLocaleState] = useState<Locale>(getLocale);
  const [listenState, setListenState] = useState<ListenState | null>(null);
  const [busy, setBusy] = useState(false);
  const [errorKey, setErrorKey] = useState<CatalogKey | null>(null);
  const [opacity, setOpacity] = useState(1);
  const opacityEdited = useRef(false);
  const opening = useRef(new Set<FeatureKind>());

  // Boot: the saved language and opacity.
  useEffect(() => {
    let cancelled = false;
    getSettings()
      .then((settings) => {
        if (cancelled) return;
        setLocale(localeFromSetting(settings.language));
        setLocaleState(getLocale());
        setOpacity(settings.opacity);
      })
      .catch((error: unknown) => {
        if (!cancelled) setErrorKey(errorCopyKey(error));
      });
    return () => {
      cancelled = true;
    };
  }, []);

  // Follow the listen session. `subscribeListenState` resolves to its unlisten function
  // asynchronously, so the cleanup has to cope with running before that happens.
  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    subscribeListenState((state) => {
      setListenState(state);
      if (state.kind === "error") setErrorKey("errors.generic");
    })
      .then((stop) => {
        if (cancelled) stop();
        else unlisten = stop;
      })
      .catch((error: unknown) => {
        if (!cancelled) setErrorKey(errorCopyKey(error));
      });
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  // The push-to-ask global shortcut: open Ask exactly like the Ask button does
  // (created hidden below the hub if needed, then `promoteOverlay`; never `show()`).
  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    subscribePushToAsk(() => void openFeature("ask"))
      .then((stop) => {
        if (cancelled) stop();
        else unlisten = stop;
      })
      .catch((error: unknown) => {
        if (!cancelled) setErrorKey(errorCopyKey(error));
      });
    return () => {
      cancelled = true;
      unlisten?.();
    };
    // `openFeature` only touches refs, state setters and module constants, so the
    // first render's copy behaves exactly like any later one.
  }, []);

  // An error clears itself.
  useEffect(() => {
    if (errorKey === null) return;
    const timer = window.setTimeout(() => setErrorKey(null), ERROR_DISMISS_MS);
    return () => window.clearTimeout(timer);
  }, [errorKey]);

  // Persist the opacity once the slider settles — but not the value just loaded from settings.
  useEffect(() => {
    if (!opacityEdited.current) return;
    const timer = window.setTimeout(() => {
      saveSettings({ opacity }).catch((error: unknown) => setErrorKey(errorCopyKey(error)));
    }, OPACITY_SAVE_DELAY_MS);
    return () => window.clearTimeout(timer);
  }, [opacity]);

  const listening = isListening(listenState);

  async function toggleListen(): Promise<void> {
    if (busy) return;
    setBusy(true);
    setErrorKey(null);
    try {
      await (listening ? stopListen() : startListen());
    } catch (error) {
      setErrorKey(errorCopyKey(error));
    } finally {
      setBusy(false);
    }
  }

  async function openFeature(kind: FeatureKind): Promise<void> {
    if (opening.current.has(kind)) return; // a second click while it is being created must not make a duplicate
    opening.current.add(kind);
    try {
      await openOrFocus(kind, tauriWindows);
    } catch (error) {
      setErrorKey(errorCopyKey(error));
    } finally {
      opening.current.delete(kind);
    }
  }

  return (
    <div className="hub" data-tauri-drag-region lang={locale} style={{ opacity }}>
      {errorKey !== null ? (
        <div className="hub-error" role="alert">
          <button type="button" onClick={() => setErrorKey(null)} title={t(errorKey)}>
            {t(errorKey)}
          </button>
        </div>
      ) : (
        <>
          <button
            type="button"
            className="hub-btn"
            aria-pressed={listening}
            disabled={busy}
            title={listening ? t("listen.stop") : t("listen.start")}
            onClick={() => void toggleListen()}
          >
            {t("hub.listen")}
          </button>
          <button type="button" className="hub-btn" title={t("hub.ask")} onClick={() => void openFeature("ask")}>
            {t("hub.ask")}
          </button>
          <button type="button" className="hub-btn" title={t("hub.notes")} onClick={() => void openFeature("notes")}>
            {t("hub.notes")}
          </button>
          <button type="button" className="hub-btn" title={t("hub.settings")} onClick={() => void openFeature("settings")}>
            {t("hub.settings")}
          </button>
          <input
            type="range"
            className="hub-opacity"
            min={MIN_OPACITY}
            max={1}
            step={0.05}
            value={opacity}
            aria-label={t("settings.opacity")}
            title={t("settings.opacity")}
            onChange={(event) => {
              opacityEdited.current = true;
              setOpacity(event.currentTarget.valueAsNumber);
            }}
          />
          <button
            type="button"
            className="hub-btn"
            title={t("hub.close")}
            onClick={() => void hideAll(tauriHide).catch((error: unknown) => setErrorKey(errorCopyKey(error)))}
          >
            {t("hub.close")}
          </button>
        </>
      )}
    </div>
  );
}
