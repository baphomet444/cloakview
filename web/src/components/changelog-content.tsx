import type { Locale } from "@/site.config";
import type { Dictionary } from "@/lib/get-dictionary";
import { getChangelog } from "@/lib/changelog";

export async function ChangelogContent({ locale, copy }: { locale: Locale; copy: Dictionary["changelog"] }) {
  const { entries, isFallback } = await getChangelog();
  return <div className="space-y-6">
    {isFallback && <p role="status" className="rounded-lg border border-border bg-surface p-4 text-muted">{copy.fallback}</p>}
    <ol className="space-y-5">{entries.map((entry) => <li key={entry.version} className="rounded-xl border border-border bg-surface p-6">
      <p className="text-sm text-accent">{copy.release} {entry.version}{entry.date && <> · <time dateTime={entry.date}>{entry.date}</time></>}</p>
      <h2 className="font-display mt-2 text-xl font-semibold">{entry.title[locale]}</h2>
      <p className="mt-3 whitespace-pre-line text-muted">{entry.body[locale]}</p>
    </li>)}</ol>
  </div>;
}
