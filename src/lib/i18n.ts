import { en, type Catalog } from "./locales/en";
import { es } from "./locales/es";

export type Locale = "en" | "es";
export type CatalogKey = keyof Catalog;

const STORAGE_KEY = "cloakview-locale";
const catalogs: Record<Locale, Catalog> = { en, es };

const isLocale = (value: unknown): value is Locale => value === "en" || value === "es";

/**
 * The stored locale, or English. Defensive on purpose: `localStorage` is absent
 * outside a webview (node tests, SSR), can throw when storage is blocked, and
 * can hold a value from an older build — and `catalogs[stored]` would then be
 * `undefined`, breaking every `t()` call. Anything unusable falls back to "en".
 */
function readStoredLocale(): Locale {
  try {
    const stored: unknown = localStorage.getItem(STORAGE_KEY);
    return isLocale(stored) ? stored : "en";
  } catch {
    return "en";
  }
}

let current: Locale = readStoredLocale();

export function setLocale(locale: Locale): void {
  current = locale;
  try {
    localStorage.setItem(STORAGE_KEY, locale);
  } catch {
    // Storage unavailable: the choice still applies for this session, just not the next.
  }
}

export function getLocale(): Locale {
  return current;
}

export function t(key: CatalogKey): string {
  return catalogs[current][key];
}

/**
 * Boot wiring: after `getSettings()`, `setLocale(localeFromSetting(settings.language))`.
 * `settings.language` is a free-form string on the Rust side; only "es" selects Spanish.
 */
export function localeFromSetting(language: string): Locale {
  return language === "es" ? "es" : "en";
}
