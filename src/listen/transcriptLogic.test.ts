import { describe, expect, it } from "vitest";
import type { Turn } from "../lib/models";
import { appendTurn, type TranscriptLine } from "./transcriptLogic";

const turn = (text: string, isUserTurn = false): Turn => ({ speaker: isUserTurn ? "you" : "remote", text, isUserTurn, ts: 1 });

describe("appendTurn", () => {
  it("adds lines in order and marks whose they are", () => {
    let lines: TranscriptLine[] = [];
    lines = appendTurn(lines, turn(" What is your experience? "));
    lines = appendTurn(lines, turn("About five years.", true));
    expect(lines).toEqual([
      { id: 1, mine: false, text: "What is your experience?" },
      { id: 2, mine: true, text: "About five years." },
    ]);
  });

  it("ignores blank lines", () => {
    expect(appendTurn([], turn("   "))).toEqual([]);
  });

  it("drops the oldest lines past the limit and keeps ids unique", () => {
    let lines: TranscriptLine[] = [];
    for (const text of ["a", "b", "c", "d"]) lines = appendTurn(lines, turn(text), 3);
    expect(lines.map((line) => line.text)).toEqual(["b", "c", "d"]);
    expect(lines.map((line) => line.id)).toEqual([2, 3, 4]);
  });
});
