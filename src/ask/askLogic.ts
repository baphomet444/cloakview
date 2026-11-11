/**
 * The Ask window's state machine, free of React and Tauri so it can be unit-tested.
 *
 * `answer-delta`, `answer-done` and `error` are app-wide events (Rust `app.emit`), and
 * `error` is shared with the realtime side, so they only count while a question is in
 * flight (`thinking` or `streaming`); otherwise they leave the state untouched.
 */
import type { CatalogKey } from "../lib/i18n";

export type AskPhase = "idle" | "thinking" | "streaming" | "done" | "error";

export type AskState = {
  phase: AskPhase;
  /** What was asked last; blank when the user asked for a suggestion without typing. */
  question: string;
  answer: string;
  errorKey: CatalogKey | null;
};

export type AskAction =
  | { type: "send"; question: string }
  | { type: "delta"; text: string }
  | { type: "done" }
  | { type: "stop" }
  | { type: "error"; key: CatalogKey }
  | { type: "dismiss" };

export const ASK_IDLE: AskState = { phase: "idle", question: "", answer: "", errorKey: null };

export function isBusy(state: AskState): boolean {
  return state.phase === "thinking" || state.phase === "streaming";
}

export function askReducer(state: AskState, action: AskAction): AskState {
  if (action.type === "send") {
    return { phase: "thinking", question: action.question, answer: "", errorKey: null };
  }
  if (action.type === "dismiss") {
    return state.phase === "error" ? { ...state, phase: "done", errorKey: null } : state;
  }
  if (!isBusy(state)) return state;
  switch (action.type) {
    case "delta":
      return { ...state, phase: "streaming", answer: state.answer + action.text };
    case "done":
    case "stop":
      return { ...state, phase: "done" };
    case "error":
      return { ...state, phase: "error", errorKey: action.key };
  }
}

/** `request_answer`'s argument: the trimmed question, or none (suggest a reply from the transcript). */
export function questionArg(input: string): string | undefined {
  const question = input.trim();
  return question === "" ? undefined : question;
}
