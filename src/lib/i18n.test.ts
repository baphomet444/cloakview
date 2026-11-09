import { afterEach, describe, expect, it, vi } from "vitest";
import { en, type Catalog } from "./locales/en";
import { es } from "./locales/es";

/** Keys `reference` has that `candidate` lacks (`missing`), and keys `candidate` has that `reference` lacks (`extra`). */
function keyDiff(reference: object, candidate: object): { missing: string[]; extra: string[] } {
  const ref = new Set(Object.keys(reference));
  const cand = new Set(Object.keys(candidate));
  return {
    missing: [...ref].filter((k) => !cand.has(k)),
    extra: [...cand].filter((k) => !ref.has(k)),
  };
}

describe("i18n parity", () => {
  // The plan's test, expressed through keyDiff so a failure names the offending keys.
  it("es has exactly the same non-empty keys as en", () => {
    expect(keyDiff(en, es)).toEqual({ missing: [], extra: [] });
    for (const [key, value] of Object.entries(es)) {
      expect(value.trim().length, `es["${key}"] is empty`).toBeGreaterThan(0);
    }
    for (const [key, value] of Object.entries(en)) {
      expect(value.trim().length, `en["${key}"] is empty`).toBeGreaterThan(0);
    }
  });

  // Proves the check above is able to fail: it is not a tautology.
  it("the parity check reports a key missing from es and a key es should not have", () => {
    const withoutOne: Record<string, string> = { ...es };
    delete withoutOne["hub.close"];
    expect(keyDiff(en, withoutOne)).toEqual({ missing: ["hub.close"], extra: [] });

    const withOneMore: Record<string, string> = { ...es, "not.a.key": "x" };
    expect(keyDiff(en, withOneMore)).toEqual({ missing: [], extra: ["not.a.key"] });
  });

  // Checked by `tsc --noEmit` (the build gate), not at runtime: if `Catalog` stopped
  // rejecting a missing or an extra key, the `@ts-expect-error` lines would themselves error.
  it("es is typed as typeof en: a missing or extra key does not compile", () => {
    // @ts-expect-error a key missing from the catalog must be a type error
    const missing: Catalog = { "hub.listen": "Escuchar" };
    // @ts-expect-error a key that `en` does not have must be a type error
    const extra: Catalog = { ...es, "not.a.key": "x" };
    expect([missing, extra]).toHaveLength(2);
  });
});

/** A throwaway `Storage` stand-in; only what i18n.ts touches. */
function memoryStorage(initial: Record<string, string> = {}) {
  const data = new Map(Object.entries(initial));
  return {
    getItem: vi.fn((key: string) => data.get(key) ?? null),
    setItem: vi.fn((key: string, value: string) => void data.set(key, value)),
  };
}

/** i18n.ts keeps its locale in module state, so each test imports a fresh copy. */
async function freshI18n(): Promise<typeof import("./i18n")> {
  vi.resetModules();
  return import("./i18n");
}

describe("i18n runtime", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("loads in an environment with no localStorage (node/SSR/tests), defaulting to English", async () => {
    vi.stubGlobal("localStorage", undefined);
    const i18n = await freshI18n();
    expect(i18n.getLocale()).toBe("en");
    expect(i18n.t("hub.listen")).toBe(en["hub.listen"]);
  });

  it("setLocale switches the catalog t() reads from", async () => {
    vi.stubGlobal("localStorage", memoryStorage());
    const i18n = await freshI18n();
    i18n.setLocale("es");
    expect(i18n.getLocale()).toBe("es");
    expect(i18n.t("hub.listen")).toBe(es["hub.listen"]);
    i18n.setLocale("en");
    expect(i18n.t("hub.listen")).toBe(en["hub.listen"]);
  });

  it("restores a stored locale on load, and persists changes", async () => {
    const storage = memoryStorage({ "cloakview-locale": "es" });
    vi.stubGlobal("localStorage", storage);
    const i18n = await freshI18n();
    expect(i18n.getLocale()).toBe("es");

    i18n.setLocale("en");
    expect(storage.setItem).toHaveBeenCalledWith("cloakview-locale", "en");
  });

  // The plan blind-casts the stored string, so "fr" would leave t() reading `undefined["..."]`.
  it("ignores a corrupt stored locale instead of breaking t()", async () => {
    vi.stubGlobal("localStorage", memoryStorage({ "cloakview-locale": "fr" }));
    const i18n = await freshI18n();
    expect(i18n.getLocale()).toBe("en");
    expect(i18n.t("errors.generic")).toBe(en["errors.generic"]);
  });

  it("survives a storage that throws (blocked or private-mode storage)", async () => {
    const hostile = {
      getItem: () => {
        throw new Error("denied");
      },
      setItem: () => {
        throw new Error("denied");
      },
    };
    vi.stubGlobal("localStorage", hostile);
    const i18n = await freshI18n();
    expect(i18n.getLocale()).toBe("en");
    expect(() => i18n.setLocale("es")).not.toThrow();
    expect(i18n.t("hub.listen")).toBe(es["hub.listen"]); // still switches, in memory
  });

  it("localeFromSetting maps the backend's `language` setting: es -> es, anything else -> en", async () => {
    vi.stubGlobal("localStorage", undefined);
    const { localeFromSetting } = await freshI18n();
    expect(localeFromSetting("es")).toBe("es");
    expect(localeFromSetting("en")).toBe("en");
    expect(localeFromSetting("fr")).toBe("en");
    expect(localeFromSetting("")).toBe("en");
  });
});
