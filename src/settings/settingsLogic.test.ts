import { describe, expect, it } from "vitest";
import type { Settings } from "../lib/models";
import {
  MIN_OPACITY,
  clampOpacity,
  draftFromSettings,
  draftIsValid,
  draftPatch,
  isDraftDirty,
  maskKey,
  themeFromSetting,
} from "./settingsLogic";

const settings: Settings = {
  language: "en",
  theme: "dark",
  opacity: 0.9,
  activeProvider: "openai",
  modelOverrides: {},
  realtimeModel: "gpt-4o-mini-transcribe",
  replyDelayMs: 1000,
  userKeywords: [],
  userName: null,
  shortcuts: { quickHide: "cmdOrControl+shift+h", pushToAsk: "cmdOrControl+shift+space", toggleListen: "", readScreen: "" },
  includeMic: false,
  personalContext: "",
  answerStyle: "balanced",
  translateTo: "",
  callLanguage: "",
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

describe("maskKey", () => {
  it("hides everything but the last four characters", () => {
    expect(maskKey("ABCD-1234-EFGH-5678")).toBe("•••• 5678");
    // A key of four characters or fewer would show whole, so it shows nothing.
    expect(maskKey("ABC")).toBe("••••");
    expect(maskKey("1234")).toBe("••••");
  });
});

describe("the shortcuts and context draft", () => {
  it("starts from the saved settings and is not dirty", () => {
    const draft = draftFromSettings(settings);
    expect(draft).toEqual({ shortcuts: settings.shortcuts, personalContext: "" });
    expect(isDraftDirty(draft, settings)).toBe(false);
  });

  it("is dirty once any field differs after trimming, and not for whitespace alone", () => {
    const draft = draftFromSettings(settings);
    expect(isDraftDirty({ ...draft, shortcuts: { ...draft.shortcuts, toggleListen: "cmdOrControl+shift+l" } }, settings)).toBe(true);
    expect(isDraftDirty({ ...draft, personalContext: "  Rust engineer.  " }, { ...settings, personalContext: "Rust engineer." })).toBe(false);
    expect(isDraftDirty({ ...draft, personalContext: "Rust engineer." }, settings)).toBe(true);
  });

  it("is always valid (a blank shortcut turns that action off)", () => {
    expect(draftIsValid(draftFromSettings(settings))).toBe(true);
    expect(draftIsValid({ ...draftFromSettings(settings), shortcuts: { quickHide: "", pushToAsk: "", toggleListen: "", readScreen: "" } })).toBe(true);
  });

  it("saves trimmed values and always all four shortcuts (the patch replaces the set)", () => {
    const patch = draftPatch({
      shortcuts: { quickHide: " cmdOrControl+shift+h ", pushToAsk: "", toggleListen: "alt+l ", readScreen: " alt+s " },
      personalContext: "  Senior Rust engineer.  ",
    });
    expect(patch).toEqual({
      shortcuts: { quickHide: "cmdOrControl+shift+h", pushToAsk: "", toggleListen: "alt+l", readScreen: "alt+s" },
      personalContext: "Senior Rust engineer.",
    });
  });
});
