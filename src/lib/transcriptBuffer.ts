import type { Turn } from "./models";

/**
 * Maximum turns kept. Must equal `CAPACITY` in `src-tauri/src/transcript.rs`
 * (a test reads that file and compares).
 */
export const CAPACITY = 30;

/**
 * TypeScript mirror of the Rust transcript ring: `TranscriptBuffer` in
 * `src-tauri/src/transcript.rs`, and `push_turn` in `listen.rs`, which is what
 * actually feeds it. Same discard rule: once the buffer holds `CAPACITY` turns,
 * the oldest is dropped *before* the new one is appended, so the buffer is
 * never longer than `CAPACITY` and always keeps the most recent turns in order.
 */
export class TranscriptBuffer {
  private turns: Turn[] = [];

  push(turn: Turn): void {
    if (this.turns.length >= CAPACITY) this.turns.shift();
    this.turns.push(turn);
  }

  /**
   * The `n` most recent turns, oldest first (like Rust's `last`). `n` beyond the
   * length returns everything; `n <= 0` returns nothing — written out because
   * `slice(-0)` would return the whole array. A fresh array: callers can't
   * mutate the buffer through it.
   */
  last(n: number): Turn[] {
    return n <= 0 ? [] : this.turns.slice(-n);
  }

  len(): number {
    return this.turns.length;
  }

  isEmpty(): boolean {
    return this.turns.length === 0;
  }
}
