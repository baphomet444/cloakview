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
};

/** Adds a line at the end, dropping the oldest ones past `max`. Blank lines are ignored. */
export function appendTurn(lines: readonly TranscriptLine[], turn: Turn, max = MAX_LINES): TranscriptLine[] {
  const text = turn.text.trim();
  if (text === "") return [...lines];
  const id = lines.length === 0 ? 1 : lines[lines.length - 1].id + 1;
  return [...lines, { id, mine: turn.isUserTurn, text }].slice(-max);
}
