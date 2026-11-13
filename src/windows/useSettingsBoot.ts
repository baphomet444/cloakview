import { useEffect, useState } from "react";
import { getLocale, localeFromSetting, setLocale, type CatalogKey, type Locale } from "../lib/i18n";
import { getSettings, subscribeSettingsChanged } from "../lib/ipc";
import type { Settings } from "../lib/models";
import { errorCopyKey } from "../hub/hubLogic";
import { clampOpacity, themeFromSetting, type Theme } from "../settings/settingsLogic";

/** Puts the theme and opacity on `<html>`, where `styles.css` reads them. */
export function applyAppearance(theme: Theme, opacity: number): void {
  document.documentElement.dataset.theme = theme;
  document.documentElement.style.setProperty("--win-opacity", String(clampOpacity(opacity)));
}

/**
 * Boot for a feature window: the saved settings, with the language, theme and opacity
 * applied. Each window is its own webview, loaded once and then only shown or hidden
 * for the rest of the app's life (the hub never destroys and recreates it), so this
 * also applies the theme and opacity live on every `settings-changed` broadcast --
 * otherwise a window already open when another window saves settings would stay on
 * whatever appearance it booted with until the app restarts. Language still only
 * takes effect the next time the window loads: swapping the locale catalog live is a
 * bigger change than this fix calls for, and no one has reported it stale.
 */
export function useSettingsBoot(): {
  settings: Settings | null;
  setSettings: (settings: Settings) => void;
  locale: Locale;
  bootErrorKey: CatalogKey | null;
} {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [locale, setLocaleState] = useState<Locale>(getLocale);
  const [bootErrorKey, setBootErrorKey] = useState<CatalogKey | null>(null);

  useEffect(() => {
    let cancelled = false;
    getSettings()
      .then((loaded) => {
        if (cancelled) return;
        setLocale(localeFromSetting(loaded.language));
        setLocaleState(getLocale());
        applyAppearance(themeFromSetting(loaded.theme), loaded.opacity);
        setSettings(loaded);
      })
      .catch((error: unknown) => {
        if (!cancelled) setBootErrorKey(errorCopyKey(error));
      });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    subscribeSettingsChanged((changed) => {
      if (cancelled) return;
      applyAppearance(themeFromSetting(changed.theme), changed.opacity);
      setSettings(changed);
    })
      .then((stop) => {
        if (cancelled) stop();
        else unlisten = stop;
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  return { settings, setSettings, locale, bootErrorKey };
}
