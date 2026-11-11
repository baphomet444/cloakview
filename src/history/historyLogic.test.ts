import { describe, expect, it } from "vitest";
import type { SessionEntry } from "../lib/models";
import { elapsed, entryPrompt, entryView, nextQuery } from "./historyLogic";

const heard: SessionEntry = { kind: "heard", ts: 12_000, text: "Tell me about yourself.", mine: false };
const mine: SessionEntry = { kind: "heard", ts: 16_000, text: "Sure.", mine: true };
const suggested: SessionEntry = { kind: "suggestion", ts: 20_000, prompt: "Tell me about yourself.", text: "I led a rewrite." };
const unprompted: SessionEntry = { kind: "suggestion", ts: 21_000, prompt: "  ", text: "A." };

describe("entryView", () => {
  it("labels the call's lines as theirs, the user's own as yours, and suggestions as suggestions", () => {
    expect(entryView(heard)).toEqual({ labelKey: "history.theySaid", text: "Tell me about yourself." });
    expect(entryView(mine)).toEqual({ labelKey: "history.youSaid", text: "Sure." });
    expect(entryView(suggested)).toEqual({ labelKey: "ask.suggestion", text: "I led a rewrite." });
  });
});

describe("entryPrompt", () => {
  it("shows the prompt only when there is one", () => {
    expect(entryPrompt(suggested)).toBe("Tell me about yourself.");
    expect(entryPrompt(unprompted)).toBeNull();
    expect(entryPrompt(heard)).toBeNull();
  });
});

describe("nextQuery", () => {
  it("blank and whitespace mean 'list everything'", () => {
    expect(nextQuery("")).toBeUndefined();
    expect(nextQuery("   ")).toBeUndefined();
    expect(nextQuery(" kubernetes ")).toBe("kubernetes");
  });
});

describe("elapsed", () => {
  it("matches the .txt export's mm:ss / h:mm:ss shape", () => {
    expect(elapsed(0)).toBe("00:00");
    expect(elapsed(65_000)).toBe("01:05");
    expect(elapsed(3_725_000)).toBe("1:02:05");
    expect(elapsed(-5)).toBe("00:00");
  });
});
