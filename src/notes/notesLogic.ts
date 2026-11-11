/** The Notes window's decisions, free of React and Tauri so they can be unit-tested. */
import type { NewNote, Note } from "../lib/models";

export function canCreateNote(title: string, content: string): boolean {
  return title.trim() !== "" || content.trim() !== "";
}

/** What `create_note` receives: the title trimmed, the content without outer blank lines (inner indentation kept). */
export function newNoteInput(title: string, content: string): NewNote {
  return { title: title.trim(), content: content.replace(/^\s*\n/, "").trimEnd() };
}

/** The line a note is listed under: its title, else its first non-blank line, else null (the UI's "untitled" copy). */
export function noteHeading(note: Note): string | null {
  const title = note.title.trim();
  if (title !== "") return title;
  const firstLine = note.content
    .split("\n")
    .map((line) => line.trim())
    .find((line) => line !== "");
  return firstLine ?? null;
}

/** The list after `create_note` resolved: newest first, as `list_notes` orders it. */
export function withCreated(notes: readonly Note[], created: Note): Note[] {
  return [created, ...notes.filter((n) => n.id !== created.id)];
}

export function withoutNote(notes: readonly Note[], id: string): Note[] {
  return notes.filter((n) => n.id !== id);
}
