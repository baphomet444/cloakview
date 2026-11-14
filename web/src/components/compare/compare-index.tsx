import Link from "next/link";
import type { Locale } from "@/site.config";
import {
  CHECKED_AT,
  compareDocs,
  compareIndex,
  compareLabels,
  comparePath,
  compareSlugs,
} from "@/content/compare";

export function CompareIndex({ locale }: { locale: Locale }) {
  const copy = compareIndex[locale];
  const labels = compareLabels[locale];

  return (
    <div className="mx-auto max-w-4xl px-6 py-14">
      <h1 className="font-display text-4xl font-semibold tracking-tight text-foreground sm:text-5xl">
        {copy.h1}
      </h1>
      <p className="mt-5 text-lg leading-relaxed text-muted">{copy.intro}</p>
      <p className="mt-4 rounded-xl border border-border bg-surface p-5 text-sm leading-relaxed text-muted">
        {copy.note}
      </p>

      <ul className="mt-10 space-y-4">
        {compareSlugs.map((slug) => {
          const doc = compareDocs[locale][slug];
          return (
            <li key={slug}>
              <Link
                href={comparePath(locale, slug)}
                className="block rounded-xl border border-border bg-surface p-6 hover:border-accent"
              >
                <span className="font-display text-xl font-semibold text-foreground">
                  CloakView vs {doc.name}
                </span>
                <span className="mt-2 block text-muted">{doc.standfirst}</span>
                <span className="mt-3 block text-sm text-accent">{labels.readLabel} →</span>
              </Link>
            </li>
          );
        })}
      </ul>

      <p className="mt-10 text-sm text-muted">
        {copy.checkedLabel}: {CHECKED_AT[locale]}
      </p>
    </div>
  );
}
