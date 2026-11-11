import { describe, expect, it } from "vitest";
import type { Settings } from "../lib/models";
import {
  MIN_OPACITY,
  clampOpacity,
  draftFromSettings,
  draftIsValid,
  draftPatch,
  isDraftDirty,
  themeFromSetting,
} from "./settingsLogic";

const settings: Settings = {
  language: "en",
  theme: "dark",
  opacity: 0.9,
  answersModel: "gpt-4o-mini",
  realtimeModel: "gpt-4o-realtime-preview",
  userKeywords: [],
  userName: null,
  shortcuts: { quickHide: "cmdOrControl+shift+h", pushToAsk: "cmdOrControl+shift+space", toggleListen: "" },
};

describe("themeFromSetting", () => {
  it("only 'light' selects the light theme; anything else, including an unknown value, is dark", () => {
    expect(themeFromSetting("light")).toBe("light");
    expect(themeFromSetting("dark")).toBe("dark");
    expect(themeFromSetting("system")).toBe("dark");
    expect(themeFromSetting("")).toBe("dark");
  });
});

describe("clampOpacity", () => {
  it("keeps the overlay findable and never above fully opaque", () => {
    expect(clampOpacity(0)).toBe(MIN_OPACITY);
    expect(clampOpacity(0.5)).toBe(0.5);
    expect(clampOpacity(3)).toBe(1);
    expect(clampOpacity(Number.NaN)).toBe(1);
  });
});

describe("the model and shortcuts draft", () => {
  it("starts from the saved settings and is not dirty", () => {
    const draft = draftFromSettings(settings);
    expect(draft).toEqual({ answersModel: "gpt-4o-mini", shortcuts: settings.shortcuts });
    expect(isDraftDirty(draft, settings)).toBe(false);
  });

  it("is dirty once any field differs after trimming, and not for whitespace alone", () => {
    const draft = draftFromSettings(settings);
    expect(isDraftDirty({ ...draft, answersModel: " gpt-4o-mini " }, settings)).toBe(false);
    expect(isDraftDirty({ ...draft, answersModel: "gpt-4.1" }, settings)).toBe(true);
    expect(isDraftDirty({ ...draft, shortcuts: { ...draft.shortcuts, toggleListen: "cmdOrControl+shift+l" } }, settings)).toBe(true);
  });

  it("is invalid with a blank model (Rust would accept it and every answer would fail)", () => {
    expect(draftIsValid(draftFromSettings(settings))).toBe(true);
    expect(draftIsValid({ ...draftFromSettings(settings), answersModel: "   " })).toBe(false);
  });

  it("a blank shortcut is valid: it turns that action off", () => {
    expect(draftIsValid({ ...draftFromSettings(settings), shortcuts: { quickHide: "", pushToAsk: "", toggleListen: "" } })).toBe(true);
  });

  it("saves trimmed values and always all three shortcuts (the patch replaces the set)", () => {
    const patch = draftPatch({
      answersModel: "  gpt-4.1 ",
      shortcuts: { quickHide: " cmdOrControl+shift+h ", pushToAsk: "", toggleListen: "alt+l " },
    });
    expect(patch).toEqual({
      answersModel: "gpt-4.1",
      shortcuts: { quickHide: "cmdOrControl+shift+h", pushToAsk: "", toggleListen: "alt+l" },
    });
  });
});
