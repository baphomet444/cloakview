import "server-only";
import type { Locale } from "@/site.config";

const dictionaries = {
  en: () => import("@/dictionaries/en.json").then((m) => m.default),
  es: () => import("@/dictionaries/es.json").then((m) => m.default),
};

export async function getDictionary(locale: Locale) {
  const load = dictionaries[locale] ?? dictionaries.en;
  return load();
}

export type Dictionary = Awaited<ReturnType<typeof getDictionary>>;
