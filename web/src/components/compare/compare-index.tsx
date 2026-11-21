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

      <section aria-labelledby="quick-answer" className="mt-12">
        <h2 id="quick-answer" className="font-display text-2xl font-semibold tracking-tight text-foreground">
          {copy.quickHeading}
        </h2>
        <div className="mt-4 space-y-4 text-lg leading-relaxed text-muted">
          {copy.quickAnswer.map((paragraph) => (
            <p key={paragraph.slice(0, 32)}>{paragraph}</p>
          ))}
        </div>
      </section>

      <ul className="mt-10 space-y-4">
        {compareSlugs.map((slug) => {
          const doc = compareDocs[locale][slug];
          return (
            <li key={slug}>
              <Link
                href={comparePath(locale, slug)}
                className="block rounded-xl border border-border bg-surface p-6 hover:border-accent"
              >
                <h2 className="font-display text-xl font-semibold text-foreground">
                  CloakView vs {doc.name}
                </h2>
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

      <section aria-labelledby="compare-faq" className="mt-12">
        <h2 id="compare-faq" className="font-display text-2xl font-semibold tracking-tight text-foreground">
          {labels.faqHeading}
        </h2>
        <div className="mt-5 space-y-3">
          {copy.faq.map((item) => (
            <details key={item.q} className="rounded-xl border border-border bg-surface px-5 py-4">
              <summary className="cursor-pointer font-medium text-foreground">{item.q}</summary>
              <p className="mt-3 text-muted">{item.a}</p>
            </details>
          ))}
        </div>
      </section>
    </div>
  );
}
