import { describe, expect, it } from "vitest";
import type { Turn } from "../lib/models";
import { appendTurn, withTranslation, type TranscriptLine } from "./transcriptLogic";

const turn = (text: string, isUserTurn = false): Turn => ({ speaker: isUserTurn ? "you" : "remote", text, isUserTurn, ts: 1 });

describe("appendTurn", () => {
  it("adds lines in order and marks whose they are", () => {
    let lines: TranscriptLine[] = [];
    lines = appendTurn(lines, { ...turn(" What is your experience? "), ts: 100 });
    lines = appendTurn(lines, { ...turn("About five years.", true), ts: 200 });
    expect(lines).toEqual([
      { id: 1, mine: false, text: "What is your experience?", ts: 100 },
      { id: 2, mine: true, text: "About five years.", ts: 200 },
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

describe("withTranslation", () => {
  it("records translations by their line's ts, replacing an earlier one", () => {
    let map = withTranslation(new Map(), { ts: 100, text: "hola" });
    map = withTranslation(map, { ts: 200, text: "buenos días" });
    expect(map.get(100)).toBe("hola");
    map = withTranslation(map, { ts: 100, text: "hello" });
    expect(map.get(100)).toBe("hello");
    expect(map.size).toBe(2);
  });
});
