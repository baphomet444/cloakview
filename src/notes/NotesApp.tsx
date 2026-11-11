import { useEffect, useState } from "react";
import { getLocale, t, type CatalogKey } from "../lib/i18n";
import { createNote, deleteNote, listNotes } from "../lib/ipc";
import type { Note } from "../lib/models";
import { errorCopyKey } from "../hub/hubLogic";
import { WindowFrame } from "../windows/WindowFrame";
import { useSettingsBoot } from "../windows/useSettingsBoot";
import { canCreateNote, newNoteInput, noteHeading, withCreated, withoutNote } from "./notesLogic";

/** When a note was last changed, in the window's language (`Intl`, no fixed format). */
function formatWhen(ms: number): string {
  return new Intl.DateTimeFormat(getLocale(), { dateStyle: "medium", timeStyle: "short" }).format(ms);
}

/** Write a note, see every note (newest first) and delete one. */
export function NotesApp() {
  const { locale, bootErrorKey } = useSettingsBoot();
  /** `null` while the first `list_notes` is in flight, or if it failed. */
  const [notes, setNotes] = useState<Note[] | null>(null);
  const [loadFailed, setLoadFailed] = useState(false);
  const [title, setTitle] = useState("");
  const [content, setContent] = useState("");
  const [busy, setBusy] = useState(false);
  const [errorKey, setErrorKey] = useState<CatalogKey | null>(null);

  useEffect(() => {
    if (bootErrorKey !== null) setErrorKey(bootErrorKey);
  }, [bootErrorKey]);

  useEffect(() => {
    let cancelled = false;
    listNotes()
      .then((loaded) => {
        if (!cancelled) setNotes(loaded);
      })
      .catch((error: unknown) => {
        if (cancelled) return;
        setLoadFailed(true);
        setErrorKey(errorCopyKey(error));
      });
    return () => {
      cancelled = true;
    };
  }, []);

  async function create(): Promise<void> {
    if (busy || !canCreateNote(title, content)) return;
    setBusy(true);
    try {
      const created = await createNote(newNoteInput(title, content));
      setNotes((current) => withCreated(current ?? [], created));
      setTitle("");
      setContent("");
    } catch (error) {
      setErrorKey(errorCopyKey(error));
    } finally {
      setBusy(false);
    }
  }

  async function remove(id: string): Promise<void> {
    try {
      await deleteNote(id);
      setNotes((current) => (current === null ? current : withoutNote(current, id)));
    } catch (error) {
      setErrorKey(errorCopyKey(error));
    }
  }

  return (
    <WindowFrame titleKey="notes.title" locale={locale} errorKey={errorKey} onDismissError={() => setErrorKey(null)}>
      <form
        className="notes-new"
        onSubmit={(event) => {
          event.preventDefault();
          void create();
        }}
      >
        <h2 className="win-label">{t("notes.new")}</h2>
        <input
          className="win-input"
          type="text"
          value={title}
          placeholder={t("notes.titlePlaceholder")}
          aria-label={t("notes.titlePlaceholder")}
          onChange={(event) => setTitle(event.currentTarget.value)}
        />
        <textarea
          className="win-input win-textarea"
          value={content}
          rows={3}
          placeholder={t("notes.contentPlaceholder")}
          aria-label={t("notes.contentPlaceholder")}
          onChange={(event) => setContent(event.currentTarget.value)}
        />
        <div className="win-row win-row-end">
          <button type="submit" className="win-btn win-btn-primary" disabled={busy || !canCreateNote(title, content)}>
            {t("notes.save")}
          </button>
        </div>
      </form>

      {notes === null ? (
        !loadFailed && <p className="win-muted">{t("notes.loading")}</p>
      ) : notes.length === 0 ? (
        <p className="win-empty">{t("notes.empty")}</p>
      ) : (
        <ul className="notes-list">
          {notes.map((note) => {
            const heading = noteHeading(note);
            return (
              <li key={note.id} className="notes-item">
                <div className="notes-text">
                  <h3 className={heading === null ? "notes-heading win-muted" : "notes-heading"}>
                    {heading ?? t("notes.untitled")}
                  </h3>
                  {note.title.trim() !== "" && note.content.trim() !== "" && (
                    <p className="notes-content">{note.content}</p>
                  )}
                  <time className="notes-when" dateTime={new Date(note.updatedAt).toISOString()}>
                    {formatWhen(note.updatedAt)}
                  </time>
                </div>
                <button type="button" className="win-btn win-btn-quiet" onClick={() => void remove(note.id)}>
                  {t("notes.delete")}
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </WindowFrame>
  );
}
