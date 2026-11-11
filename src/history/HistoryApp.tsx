import { useEffect, useState } from "react";
import { getLocale, t, type CatalogKey } from "../lib/i18n";
import { deleteSession, getSession, listSessions } from "../lib/ipc";
import type { PastSession, PastSessionDetail } from "../lib/models";
import { errorCopyKey } from "../hub/hubLogic";
import { WindowFrame } from "../windows/WindowFrame";
import { useSettingsBoot } from "../windows/useSettingsBoot";
import { elapsed, entryPrompt, entryView, nextQuery } from "./historyLogic";

/** When an interview happened, in the window's language (`Intl`, no fixed format). */
function formatWhen(ms: number): string {
  return new Intl.DateTimeFormat(getLocale(), { dateStyle: "medium", timeStyle: "short" }).format(ms);
}

/** Search past interviews, open one to read it whole, delete one. CloakView Pro. */
export function HistoryApp() {
  const { locale, bootErrorKey } = useSettingsBoot();
  /** `null` while the first `list_sessions` is in flight, or if it failed. */
  const [summaries, setSummaries] = useState<PastSession[] | null>(null);
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState<PastSessionDetail | null>(null);
  const [loadFailed, setLoadFailed] = useState(false);
  const [errorKey, setErrorKey] = useState<CatalogKey | null>(null);

  useEffect(() => {
    if (bootErrorKey !== null) setErrorKey(bootErrorKey);
  }, [bootErrorKey]);

  useEffect(() => {
    let cancelled = false;
    listSessions()
      .then((loaded) => {
        if (!cancelled) setSummaries(loaded);
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

  async function search(input: string): Promise<void> {
    try {
      setSummaries(await listSessions(nextQuery(input)));
    } catch (error) {
      setErrorKey(errorCopyKey(error));
    }
  }

  async function openDetail(id: string): Promise<void> {
    try {
      setOpen(await getSession(id));
    } catch (error) {
      setErrorKey(errorCopyKey(error));
    }
  }

  async function remove(id: string): Promise<void> {
    try {
      await deleteSession(id);
      setSummaries((current) => (current === null ? current : current.filter((s) => s.id !== id)));
    } catch (error) {
      setErrorKey(errorCopyKey(error));
    }
  }

  return (
    <WindowFrame titleKey="hub.history" locale={locale} errorKey={errorKey} onDismissError={() => setErrorKey(null)}>
      {open !== null ? (
        <div className="history-detail">
          <div className="win-row">
            <button type="button" className="win-btn" onClick={() => setOpen(null)}>
              {t("history.back")}
            </button>
            <h2 className="win-label">{open.title}</h2>
          </div>
          <ul className="history-entries">
            {open.entries.map((entry, index) => {
              const view = entryView(entry);
              const prompt = entryPrompt(entry);
              return (
                <li key={index} className={`history-entry history-${view.labelKey === "ask.suggestion" ? "suggestion" : "line"}`}>
                  <span className="history-elapsed">{elapsed(entry.ts - open.startedAt)}</span>
                  <div className="history-text">
                    <h3 className="history-who">{t(view.labelKey)}</h3>
                    {prompt !== null && <p className="history-prompt">{prompt}</p>}
                    <p>{view.text}</p>
                  </div>
                </li>
              );
            })}
          </ul>
        </div>
      ) : (
        <>
          <form
            className="win-row"
            onSubmit={(event) => {
              event.preventDefault();
              void search(query);
            }}
          >
            <input
              className="win-input"
              type="search"
              value={query}
              placeholder={t("history.searchPlaceholder")}
              aria-label={t("history.searchPlaceholder")}
              onChange={(event) => setQuery(event.currentTarget.value)}
            />
            <button type="submit" className="win-btn win-btn-primary">
              {t("history.search")}
            </button>
          </form>

          {summaries === null ? (
            !loadFailed && <p className="win-muted">{t("history.loading")}</p>
          ) : summaries.length === 0 ? (
            <p className="win-empty">{t("history.empty")}</p>
          ) : (
            <ul className="notes-list">
              {summaries.map((summary) => (
                <li key={summary.id} className="notes-item">
                  <div className="notes-text">
                    <h3 className="notes-heading">
                      <button type="button" className="history-open" onClick={() => void openDetail(summary.id)}>
                        {summary.title}
                      </button>
                    </h3>
                    <p className="notes-content">
                      {summary.heard} {t("history.lines")} · {summary.suggestions} {t("history.suggestedCount")}
                    </p>
                    <time className="notes-when" dateTime={new Date(summary.startedAt).toISOString()}>
                      {formatWhen(summary.startedAt)}
                    </time>
                  </div>
                  <button type="button" className="win-btn win-btn-quiet" onClick={() => void remove(summary.id)}>
                    {t("notes.delete")}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </>
      )}
    </WindowFrame>
  );
}
