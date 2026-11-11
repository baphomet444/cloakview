import { describe, expect, it } from "vitest";
import { ASK_IDLE, askReducer, isBusy, questionArg, type AskState } from "./askLogic";

const thinking: AskState = { phase: "thinking", question: "q", answer: "", errorKey: null };
const streaming: AskState = { phase: "streaming", question: "q", answer: "Hel", errorKey: null };

describe("askReducer", () => {
  it("send clears the previous answer and error, and waits for the first delta", () => {
    const before: AskState = { phase: "error", question: "old", answer: "partial", errorKey: "errors.generic" };
    expect(askReducer(before, { type: "send", question: "new" })).toEqual({
      phase: "thinking",
      question: "new",
      answer: "",
      errorKey: null,
    });
  });

  it("deltas append in order and switch thinking to streaming", () => {
    const one = askReducer(thinking, { type: "delta", text: "Hel" });
    const two = askReducer(one, { type: "delta", text: "lo" });
    expect(one.phase).toBe("streaming");
    expect(two).toMatchObject({ phase: "streaming", answer: "Hello" });
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
    ["done", { phase: "done", question: "q", answer: "final", errorKey: null }],
  ])("ignores deltas, done and errors when %s", (_label, state) => {
    expect(askReducer(state, { type: "delta", text: "late" })).toBe(state);
    expect(askReducer(state, { type: "done" })).toBe(state);
    expect(askReducer(state, { type: "error", key: "errors.generic" })).toBe(state);
    expect(askReducer(state, { type: "stop" })).toBe(state);
  });

  it("dismiss clears the error and keeps the partial answer; it does nothing without an error", () => {
    const failed: AskState = { phase: "error", question: "q", answer: "Hel", errorKey: "errors.generic" };
    expect(askReducer(failed, { type: "dismiss" })).toEqual({ phase: "done", question: "q", answer: "Hel", errorKey: null });
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
