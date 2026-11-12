/** The live transcript's list, free of React and Tauri so it can be unit-tested. */
import type { Turn } from "../lib/models";

/** Enough for a long interview without the window growing without limit. */
export const MAX_LINES = 300;

export type TranscriptLine = {
  /** Stable React key: arrival order. */
  id: number;
  /** The user's own words (microphone) rather than the other side's. */
  mine: boolean;
  text: string;
  /** The turn's timestamp: a live translation arrives matched by it. */
  ts: number;
};

/** Adds a line at the end, dropping the oldest ones past `max`. Blank lines are ignored. */
export function appendTurn(lines: readonly TranscriptLine[], turn: Turn, max = MAX_LINES): TranscriptLine[] {
  const text = turn.text.trim();
  if (text === "") return [...lines];
  const id = lines.length === 0 ? 1 : lines[lines.length - 1].id + 1;
  return [...lines, { id, mine: turn.isUserTurn, text, ts: turn.ts }].slice(-max);
}

/** A line's translation, kept beside the line it belongs to. */
export type TranslationMap = ReadonlyMap<number, string>;

/** Records a translation under its line's `ts`. A translation for a line that has
 *  scrolled out of the window is kept but never rendered — harmless. */
export function withTranslation(
  translations: TranslationMap,
  translation: { ts: number; text: string },
): TranslationMap {
  return new Map(translations).set(translation.ts, translation.text);
}
