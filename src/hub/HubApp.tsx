import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";
import { getLocale, localeFromSetting, setLocale, t, type CatalogKey, type Locale } from "../lib/i18n";
import { getSettings, saveSettings, startListen, stopListen, subscribeListenState } from "../lib/ipc";
import type { ListenState } from "../lib/models";
import {
  FEATURE_WINDOWS,
  errorCopyKey,
  featureWindowOptions,
  isListening,
  openOrFocus,
  type FeatureKind,
  type WindowApi,
} from "./hubLogic";

/** How long an error replaces the bar before it goes away on its own. */
const ERROR_DISMISS_MS = 6000;
/** Wait for the slider to settle before writing `settings.json`, instead of on every tick. */
const OPACITY_SAVE_DELAY_MS = 250;
/** Rust clamps to 0.1..=1.0 as well; below 0.1 the hub would be invisible and impossible to find. */
const MIN_OPACITY = 0.1;

/** The real Tauri windows behind `openOrFocus`. */
const tauriWindows: WindowApi = {
  find: (label) => WebviewWindow.getByLabel(label),
  create: (kind) =>
    new Promise<void>((resolve, reject) => {
      const created = new WebviewWindow(kind, featureWindowOptions(kind, t(FEATURE_WINDOWS[kind].titleKey)));
      void created.once("tauri://created", () => resolve());
      void created.once("tauri://error", (event) => reject(new Error(String(event.payload))));
    }),
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
          <button type="button" className="hub-btn" onClick={() => void openFeature("ask")}>
            {t("hub.ask")}
          </button>
          <button type="button" className="hub-btn" onClick={() => void openFeature("notes")}>
            {t("hub.notes")}
          </button>
          <button type="button" className="hub-btn" onClick={() => void openFeature("settings")}>
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
            onClick={() => void getCurrentWindow().hide().catch((error: unknown) => setErrorKey(errorCopyKey(error)))}
          >
            {t("hub.close")}
          </button>
        </>
      )}
    </div>
  );
}
