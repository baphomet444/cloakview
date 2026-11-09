import { describe, expect, it } from "vitest";
import rustTranscript from "../../src-tauri/src/transcript.rs?raw";
import type { Turn } from "./models";
import { CAPACITY, TranscriptBuffer } from "./transcriptBuffer";

const turn = (i: number): Turn => ({ speaker: "s", text: `t${i}`, isUserTurn: false, ts: i });

function filled(count: number): TranscriptBuffer {
  const buffer = new TranscriptBuffer();
  for (let i = 0; i < count; i++) buffer.push(turn(i));
  return buffer;
}

describe("TranscriptBuffer", () => {
  // The plan's own test.
  it("bounds to last 30 turns", () => {
    const b = new TranscriptBuffer();
    for (let i = 0; i < 35; i++) b.push({ speaker: "s", text: `t${i}`, isUserTurn: false, ts: i });
    expect(b.len()).toBe(30);
    expect(b.last(30)[0].text).toBe("t5");
  });

  // Mirrors Rust's `transcript::tests::buffer_bounded_to_capacity`.
  it("mirrors the Rust ring: newest is last(1), oldest retained is t5", () => {
    const b = filled(35);
    expect(b.last(1)[0].text).toBe("t34");
    expect(b.last(30)[0].text).toBe("t5");
  });

  it("holds exactly CAPACITY turns without dropping any, and drops the oldest on the next push", () => {
    const b = filled(CAPACITY);
    expect(b.len()).toBe(CAPACITY);
    expect(b.last(CAPACITY)[0].text).toBe("t0"); // nothing evicted yet: no off-by-one

    b.push(turn(CAPACITY));
    expect(b.len()).toBe(CAPACITY);
    expect(b.last(CAPACITY)[0].text).toBe("t1"); // exactly the oldest went
  });

  it("keeps chronological order across evictions", () => {
    const b = filled(CAPACITY + 7);
    const texts = b.last(CAPACITY).map((t) => t.text);
    expect(texts).toEqual(Array.from({ length: CAPACITY }, (_, i) => `t${i + 7}`));
  });

  it("last(n) returns the n most recent in order; n beyond the length returns all; 0 returns none", () => {
    const b = filled(5);
    expect(b.last(2).map((t) => t.text)).toEqual(["t3", "t4"]);
    expect(b.last(100).map((t) => t.text)).toEqual(["t0", "t1", "t2", "t3", "t4"]);
    // `slice(-0)` is the whole array in JS: this is the trap `last(0)` must avoid.
    expect(b.last(0)).toEqual([]);
    expect(new TranscriptBuffer().last(3)).toEqual([]);
  });

  it("hands out a copy: mutating what last() returns never changes the buffer", () => {
    const b = filled(3);
    b.last(3).pop();
    b.last(3).length = 0;
    expect(b.len()).toBe(3);
    expect(b.last(3).map((t) => t.text)).toEqual(["t0", "t1", "t2"]);
  });

  it("starts empty", () => {
    const b = new TranscriptBuffer();
    expect(b.len()).toBe(0);
    expect(b.isEmpty()).toBe(true);
    b.push(turn(0));
    expect(b.isEmpty()).toBe(false);
  });

  it("uses the same capacity as the Rust buffer (transcript.rs CAPACITY)", () => {
    const rustCapacity = /pub const CAPACITY: usize = (\d+);/.exec(rustTranscript)?.[1];
    expect(rustCapacity, "CAPACITY not found in transcript.rs").toBeDefined();
    expect(CAPACITY).toBe(Number(rustCapacity));
  });
});
