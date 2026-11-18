import { describe, expect, it } from "vitest";
import {
  ASK_IDLE,
  askReducer,
  isBusy,
  isNearBottom,
  MAX_EARLIER,
  questionArg,
  shouldQueueHeard,
  type AskState,
} from "./askLogic";

const thinking: AskState = { ...ASK_IDLE, phase: "thinking", question: "q" };
const streaming: AskState = { ...ASK_IDLE, phase: "streaming", question: "q", answer: "Hel" };

describe("askReducer", () => {
  it("send starts a new answer below and keeps the previous one on screen", () => {
    const before: AskState = { ...ASK_IDLE, phase: "error", question: "old", answer: "partial", errorKey: "errors.generic" };
    expect(askReducer(before, { type: "send", question: "new", source: "heard" })).toEqual({
      phase: "thinking",
      source: "heard",
      question: "new",
      answer: "",
      errorKey: null,
      earlier: [{ source: "typed", question: "old", answer: "partial" }],
    });
  });

  it("a run of questions builds a thread instead of rewriting one answer", () => {
    let state = askReducer(ASK_IDLE, { type: "send", question: "Tell me about yourself.", source: "heard" });
    state = askReducer(state, { type: "delta", text: "I build payment systems." });
    state = askReducer(state, { type: "done" });
    state = askReducer(state, { type: "send", question: "Why this company?", source: "heard" });
    state = askReducer(state, { type: "delta", text: "Your mission." });
    expect(state.earlier).toEqual([
      { source: "heard", question: "Tell me about yourself.", answer: "I build payment systems." },
    ]);
    expect(state).toMatchObject({ question: "Why this company?", answer: "Your mission." });
  });

  it("nothing empty is kept, and only the newest MAX_EARLIER stay", () => {
    expect(askReducer(ASK_IDLE, { type: "send", question: "q" }).earlier).toEqual([]);
    let state: AskState = ASK_IDLE;
    for (let i = 0; i < MAX_EARLIER + 5; i++) {
      state = askReducer(state, { type: "send", question: `q${i}` });
      state = askReducer(state, { type: "done" });
    }
    expect(state.earlier).toHaveLength(MAX_EARLIER);
    expect(state.earlier[0].question).toBe("q4");
  });

  it("deltas append in order and switch thinking to streaming", () => {
    const one = askReducer(thinking, { type: "delta", text: "Hel" });
    const two = askReducer(one, { type: "delta", text: "lo" });
    expect(one.phase).toBe("streaming");
    expect(two).toMatchObject({ phase: "streaming", answer: "Hello" });
  });

  it("prompt shows every question the suggestion in flight answers, and is ignored once it ended", () => {
    const run = "Tell me about yourself.\nWhy this company?";
    expect(askReducer(thinking, { type: "prompt", question: run })).toEqual({ ...thinking, source: "heard", question: run });
    expect(askReducer(streaming, { type: "prompt", question: run })).toMatchObject({ question: run, answer: "Hel" });
    const done: AskState = { ...streaming, phase: "done" };
    expect(askReducer(done, { type: "prompt", question: run })).toBe(done);
  });

  it("done ends the stream and keeps the answer", () => {
    expect(askReducer(streaming, { type: "done" })).toMatchObject({ phase: "done", answer: "Hel" });
  });

  it("stop ends the stream early and keeps what arrived", () => {
    expect(askReducer(streaming, { type: "stop" })).toMatchObject({ phase: "done", answer: "Hel" });
  });

  it("an error while busy shows its copy and keeps the partial answer", () => {
    expect(askReducer(streaming, { type: "error", key: "errors.missingApiKey" })).toMatchObject({
      phase: "error",
      answer: "Hel",
      errorKey: "errors.missingApiKey",
    });
  });

  // `answer-delta`/`answer-done`/`error` are app-wide events: a late delta after Stop, or a
  // realtime error while nothing was asked, must not rewrite what the window shows.
  it.each<[string, AskState]>([
    ["idle", ASK_IDLE],
    ["done", { ...ASK_IDLE, phase: "done", question: "q", answer: "final" }],
  ])("ignores deltas, done and errors when %s", (_label, state) => {
    expect(askReducer(state, { type: "delta", text: "late" })).toBe(state);
    expect(askReducer(state, { type: "done" })).toBe(state);
    expect(askReducer(state, { type: "error", key: "errors.generic" })).toBe(state);
    expect(askReducer(state, { type: "stop" })).toBe(state);
  });

  it("dismiss clears the error and keeps the partial answer; it does nothing without an error", () => {
    const failed: AskState = { ...ASK_IDLE, phase: "error", question: "q", answer: "Hel", errorKey: "errors.generic" };
    expect(askReducer(failed, { type: "dismiss" })).toEqual({ ...ASK_IDLE, phase: "done", question: "q", answer: "Hel" });
    expect(askReducer(streaming, { type: "dismiss" })).toBe(streaming);
    expect(askReducer(ASK_IDLE, { type: "dismiss" })).toBe(ASK_IDLE);
  });

  it("a command error (send rejected) is shown even though nothing streamed yet", () => {
    expect(askReducer(thinking, { type: "error", key: "errors.missingApiKey" }).phase).toBe("error");
  });
});

describe("isBusy", () => {
  it.each<[AskState["phase"], boolean]>([
    ["idle", false],
    ["thinking", true],
    ["streaming", true],
    ["done", false],
    ["error", false],
  ])("%s -> %s", (phase, busy) => {
    expect(isBusy({ ...ASK_IDLE, phase })).toBe(busy);
  });
});

describe("questionArg", () => {
  it("sends the trimmed question", () => {
    expect(questionArg("  what now?  ")).toBe("what now?");
  });

  it("sends nothing for a blank input, so Rust suggests a reply from the transcript alone", () => {
    expect(questionArg("   ")).toBeUndefined();
    expect(questionArg("")).toBeUndefined();
  });
});

describe("shouldQueueHeard", () => {
  it("a question heard while a suggestion from the call is being written waits for it", () => {
    expect(shouldQueueHeard({ ...streaming, source: "heard" })).toBe(true);
    expect(shouldQueueHeard({ ...thinking, source: "heard" })).toBe(true);
  });

  it("otherwise it starts right away", () => {
    expect(shouldQueueHeard({ ...streaming, phase: "done", source: "heard" })).toBe(false);
    expect(shouldQueueHeard({ ...streaming, source: "typed" })).toBe(false);
    expect(shouldQueueHeard(ASK_IDLE)).toBe(false);
  });
});

describe("isNearBottom", () => {
  it("follows the answer only while the user is at the bottom", () => {
    expect(isNearBottom({ scrollTop: 600, scrollHeight: 1000, clientHeight: 400 })).toBe(true);
    expect(isNearBottom({ scrollTop: 560, scrollHeight: 1000, clientHeight: 400 })).toBe(true);
    expect(isNearBottom({ scrollTop: 200, scrollHeight: 1000, clientHeight: 400 })).toBe(false);
  });
});
