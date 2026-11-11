import { describe, expect, it } from "vitest";
import type { Note } from "../lib/models";
import { canCreateNote, newNoteInput, noteHeading, withCreated, withoutNote } from "./notesLogic";

function note(id: string, title = "", content = ""): Note {
  return { id, title, content, tags: [], favorite: false, archived: false, folderId: null, createdAt: 1, updatedAt: 1 };
}

describe("canCreateNote", () => {
  it("needs a title or some content", () => {
    expect(canCreateNote("", "")).toBe(false);
    expect(canCreateNote("  ", "\n ")).toBe(false);
    expect(canCreateNote("Title", "")).toBe(true);
    expect(canCreateNote("", "body")).toBe(true);
  });
});

describe("newNoteInput", () => {
  it("trims the title and keeps the content as typed, minus outer blank lines", () => {
    expect(newNoteInput("  Standup  ", "\n- one\n  - two\n")).toEqual({ title: "Standup", content: "- one\n  - two" });
  });
});

describe("noteHeading", () => {
  it("is the title when there is one", () => {
    expect(noteHeading(note("a", "Plan", "body"))).toBe("Plan");
  });

  it("falls back to the first line of the content", () => {
    expect(noteHeading(note("a", "  ", "\n first line \nsecond"))).toBe("first line");
  });

  it("is null for a note with neither, so the UI shows its 'untitled' copy", () => {
    expect(noteHeading(note("a", "", "  "))).toBeNull();
  });
});

describe("list updates", () => {
  it("a created note goes first (the list is newest first, like Rust's ORDER BY updated_at DESC)", () => {
    expect(withCreated([note("a"), note("b")], note("c")).map((n) => n.id)).toEqual(["c", "a", "b"]);
  });

  it("a created note that is somehow already listed is not duplicated", () => {
    expect(withCreated([note("a"), note("b")], note("b")).map((n) => n.id)).toEqual(["b", "a"]);
  });

  it("a deleted note is removed and the rest keep their order", () => {
    expect(withoutNote([note("a"), note("b"), note("c")], "b").map((n) => n.id)).toEqual(["a", "c"]);
  });
});
