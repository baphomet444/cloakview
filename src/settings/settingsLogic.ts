/** The Settings window's decisions, free of React and Tauri so they can be unit-tested. */
import type { Settings, SettingsPatch, Shortcuts } from "../lib/models";

export type Theme = "dark" | "light";

/** `settings.theme` is free-form in Rust; only "light" selects the light theme. */
export function themeFromSetting(theme: string): Theme {
  return theme === "light" ? "light" : "dark";
}

/** Rust clamps to 0.1..=1.0 as well; below 0.1 the overlay would be invisible and impossible to find. */
export const MIN_OPACITY = 0.1;

export function clampOpacity(value: number): number {
  if (!Number.isFinite(value)) return 1;
  return Math.min(1, Math.max(MIN_OPACITY, value));
}

/**
 * The fields saved with the Save button, as typed. Language, theme and opacity apply
 * (and save) the moment they change; a model name or an accelerator would be invalid
 * halfway through typing, so these wait for Save.
 */
export type SettingsDraft = {
  answersModel: string;
  shortcuts: Shortcuts;
  personalContext: string;
};

export function draftFromSettings(settings: Settings): SettingsDraft {
  return {
    answersModel: settings.answersModel,
    shortcuts: { ...settings.shortcuts },
    personalContext: settings.personalContext,
  };
}

/** Always all four shortcuts: `SettingsPatch.shortcuts` replaces the saved set. */
export function draftPatch(draft: SettingsDraft): SettingsPatch {
  return {
    answersModel: draft.answersModel.trim(),
    shortcuts: {
      quickHide: draft.shortcuts.quickHide.trim(),
      pushToAsk: draft.shortcuts.pushToAsk.trim(),
      toggleListen: draft.shortcuts.toggleListen.trim(),
      readScreen: draft.shortcuts.readScreen.trim(),
    },
    personalContext: draft.personalContext.trim(),
  };
}

/** A blank model would be saved and make every answer fail. A blank shortcut is fine: it turns that action off. */
export function draftIsValid(draft: SettingsDraft): boolean {
  return draft.answersModel.trim() !== "";
}

/** How a stored license key is shown: only its last four characters, so shoulder surfers and screenshots learn nothing. */
export function maskKey(key: string): string {
  if (key.length <= 4) return "••••";
  return `•••• ${key.slice(-4)}`;
}

export function isDraftDirty(draft: SettingsDraft, saved: Settings): boolean {
  const patch = draftPatch(draft);
  return (
    patch.answersModel !== saved.answersModel ||
    patch.personalContext !== saved.personalContext ||
    patch.shortcuts?.quickHide !== saved.shortcuts.quickHide ||
    patch.shortcuts?.pushToAsk !== saved.shortcuts.pushToAsk ||
    patch.shortcuts?.toggleListen !== saved.shortcuts.toggleListen ||
    patch.shortcuts?.readScreen !== saved.shortcuts.readScreen
  );
}
