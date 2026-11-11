import { useEffect, useState } from "react";
import { getLocale, localeFromSetting, setLocale, type CatalogKey, type Locale } from "../lib/i18n";
import { getSettings } from "../lib/ipc";
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
 * applied. Each window is its own webview, so each reads them when it loads; a change
 * made in Settings reaches another window the next time that window loads.
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

  return { settings, setSettings, locale, bootErrorKey };
}
