/** The History window's decisions, free of React and Tauri so they can be unit-tested. */
import type { SessionEntry } from "../lib/models";

/** How one entry of a past interview is shown: who spoke (a catalog key) and the text. */
export function entryView(entry: SessionEntry): {
  labelKey: "history.theySaid" | "history.youSaid" | "ask.suggestion";
  text: string;
} {
  if (entry.kind === "heard") {
    return { labelKey: entry.mine ? "history.youSaid" : "history.theySaid", text: entry.text };
  }
  return { labelKey: "ask.suggestion", text: entry.text };
}

/** The question a suggestion answers, when it has one — shown under the label. */
export function entryPrompt(entry: SessionEntry): string | null {
  return entry.kind === "suggestion" && entry.prompt.trim() !== "" ? entry.prompt : null;
}

/** What the search box sends: blank means "list everything" (`None` in Rust). */
export function nextQuery(input: string): string | undefined {
  const query = input.trim();
  return query === "" ? undefined : query;
}

/** `mm:ss`, or `h:mm:ss` from one hour on — the same shape as the .txt export (`session::elapsed`). */
export function elapsed(ms: number): string {
  const total = Math.floor(Math.max(0, ms) / 1000);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  const mm = String(m).padStart(2, "0");
  const ss = String(s).padStart(2, "0");
  return h > 0 ? `${h}:${mm}:${ss}` : `${mm}:${ss}`;
}
