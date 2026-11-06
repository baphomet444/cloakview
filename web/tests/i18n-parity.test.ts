// Test de paridad i18n — sin dependencias nuevas, usa node:test (nativo).
// Verifica que en.json y es.json tengan exactamente las mismas claves
// anidadas, para que ningún texto quede sin traducir.
//
// Ejecutar: npm run test:i18n

import { test } from "node:test";
import assert from "node:assert/strict";
import en from "../src/dictionaries/en.json" with { type: "json" };
import es from "../src/dictionaries/es.json" with { type: "json" };

function collectKeys(obj: unknown, prefix = ""): string[] {
  if (typeof obj !== "object" || obj === null) return [prefix];
  return Object.entries(obj as Record<string, unknown>).flatMap(([key, value]) =>
    collectKeys(value, prefix ? `${prefix}.${key}` : key),
  );
}

test("en.json y es.json tienen las mismas claves", () => {
  const enKeys = collectKeys(en).sort();
  const esKeys = collectKeys(es).sort();

  assert.deepEqual(
    enKeys,
    esKeys,
    `Paridad rota.\nSolo en EN: ${JSON.stringify(enKeys.filter((k) => !esKeys.includes(k)))}\nSolo en ES: ${JSON.stringify(esKeys.filter((k) => !enKeys.includes(k)))}`,
  );
});

test("ningún valor de traducción está vacío", () => {
  for (const [locale, dict] of [["en", en] as const, ["es", es] as const]) {
    const keys = collectKeys(dict);
    for (const key of keys) {
      const value = key.split(".").reduce<unknown>((acc, part) => {
        if (typeof acc === "object" && acc !== null) {
          return (acc as Record<string, unknown>)[part];
        }
        return undefined;
      }, dict);
      assert.ok(
        typeof value === "string" && value.trim().length > 0,
        `${locale}: "${key}" está vacío`,
      );
    }
  }
});
