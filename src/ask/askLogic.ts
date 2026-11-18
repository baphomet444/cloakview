/**
 * The Ask window's state machine, free of React and Tauri so it can be unit-tested.
 *
 * `answer-delta`, `answer-done` and `error` are app-wide events (Rust `app.emit`), and
 * `error` is shared with the realtime side, so they only count while a question is in
 * flight (`thinking` or `streaming`); otherwise they leave the state untouched.
 *
 * The window is a thread: each question and its answer stays on screen when the
 * next one starts (`earlier`), so a new question on the call adds its answer below
 * instead of wiping and rewriting what the user is reading.
 */
import type { CatalogKey } from "../lib/i18n";

export type AskPhase = "idle" | "thinking" | "streaming" | "done" | "error";

/** Where a question came from: typed here, heard on the call, or read from the screen. */
export type AskSource = "typed" | "heard" | "screen";

/** A finished (or cut-short) question and answer, kept above the current one. */
export type AskExchange = {
  source: AskSource;
  question: string;
  answer: string;
};

export type AskState = {
  phase: AskPhase;
  source: AskSource;
  /** What was asked last; blank when the user asked for a suggestion without typing. */
  question: string;
  answer: string;
  errorKey: CatalogKey | null;
  /** Earlier questions and answers, oldest first, at most `MAX_EARLIER`. */
  earlier: AskExchange[];
};

export type AskAction =
  | { type: "send"; question: string; source?: AskSource }
  /** What the suggestion in flight answers, as Rust worked it out (`answer-prompt`). */
  | { type: "prompt"; question: string }
  | { type: "delta"; text: string }
  | { type: "done" }
  | { type: "stop" }
  | { type: "error"; key: CatalogKey }
  | { type: "dismiss" };

/** How many earlier answers stay on screen; older ones scroll away for good. */
export const MAX_EARLIER = 20;

export const ASK_IDLE: AskState = {
  phase: "idle",
  source: "typed",
  question: "",
  answer: "",
  errorKey: null,
  earlier: [],
};

export function isBusy(state: AskState): boolean {
  return state.phase === "thinking" || state.phase === "streaming";
}

/** The current exchange moves up into `earlier` when it has anything worth keeping. */
function keepCurrent(state: AskState): AskExchange[] {
  if (state.phase === "idle" || (state.answer === "" && state.question === "")) return state.earlier;
  const kept = [...state.earlier, { source: state.source, question: state.question, answer: state.answer }];
  return kept.slice(-MAX_EARLIER);
}

export function askReducer(state: AskState, action: AskAction): AskState {
  if (action.type === "send") {
    return {
      phase: "thinking",
      source: action.source ?? "typed",
      question: action.question,
      answer: "",
      errorKey: null,
      earlier: keepCurrent(state),
    };
  }
  if (action.type === "dismiss") {
    return state.phase === "error" ? { ...state, phase: "done", errorKey: null } : state;
  }
  if (!isBusy(state)) return state;
  switch (action.type) {
    case "prompt":
      return { ...state, source: "heard", question: action.question };
    case "delta":
      return { ...state, phase: "streaming", answer: state.answer + action.text };
    case "done":
    case "stop":
      return { ...state, phase: "done" };
    case "error":
      return { ...state, phase: "error", errorKey: action.key };
  }
}

/**
 * Whether a question heard on the call should wait instead of starting now: a
 * suggestion from the call is still being written. Starting would cut it off
 * mid-sentence (Rust replaces the request in flight); waiting lets it finish, and
 * the next suggestion then answers only what came after it.
 */
export function shouldQueueHeard(state: AskState): boolean {
  return isBusy(state) && state.source === "heard";
}

/** `request_answer`'s argument: the trimmed question, or none (suggest a reply from the transcript). */
export function questionArg(input: string): string | undefined {
  const question = input.trim();
  return question === "" ? undefined : question;
}

/**
 * Whether a scroll container is at (or within `slack` pixels of) its bottom: the
 * window keeps following a streaming answer only while the user has not scrolled up.
 */
export function isNearBottom(el: { scrollTop: number; scrollHeight: number; clientHeight: number }, slack = 48): boolean {
  return el.scrollHeight - el.scrollTop - el.clientHeight <= slack;
}
