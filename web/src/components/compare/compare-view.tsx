import Link from "next/link";
import type { ReactNode } from "react";
import type { Locale } from "@/site.config";
import {
  CHECKED_AT,
  compareDocs,
  compareLabels,
  comparePath,
  compareSlugs,
  type CompareSlug,
} from "@/content/compare";

function Section({ id, title, children }: { id: string; title: string; children: ReactNode }) {
  return (
    <section aria-labelledby={id} className="mt-14">
      <h2 id={id} className="font-display text-2xl font-semibold tracking-tight text-foreground">
        {title}
      </h2>
      {children}
    </section>
  );
}

export function CompareView({ locale, slug }: { locale: Locale; slug: CompareSlug }) {
  const doc = compareDocs[locale][slug];
  const labels = compareLabels[locale];
  const others = compareSlugs.filter((s) => s !== slug);
  const themHeading = labels.theyWinHeading.replace("{name}", doc.name);

  return (
    <article className="mx-auto max-w-5xl px-6 py-14">
      <nav aria-label="Breadcrumb" className="mb-8 text-sm text-muted">
        <ol className="flex flex-wrap items-center gap-2">
          <li>
            <Link href={locale === "en" ? "/" : "/es"} className="hover:text-accent">
              {locale === "en" ? "Home" : "Inicio"}
            </Link>
          </li>
          <li aria-hidden="true">/</li>
          <li>
            <Link href={comparePath(locale)} className="hover:text-accent">
              {locale === "en" ? "Compare" : "Comparativas"}
            </Link>
          </li>
          <li aria-hidden="true">/</li>
          <li aria-current="page" className="text-foreground">
            {doc.name}
          </li>
        </ol>
      </nav>

      <h1 className="font-display text-4xl font-semibold tracking-tight text-foreground sm:text-5xl">
        {doc.h1}
      </h1>
      <p className="mt-5 max-w-3xl text-xl leading-relaxed text-muted">{doc.standfirst}</p>

      <p className="mt-6 text-sm text-muted">
        {labels.sourceLabel}{" "}
        <a
          href={doc.vendorUrl}
          target="_blank"
          rel="noopener noreferrer"
          className="text-accent underline underline-offset-4"
        >
          {doc.vendorUrl.replace(/^https:\/\//, "")}
        </a>
        {" · "}
        {CHECKED_AT[locale]}
      </p>

      <div className="mt-10 space-y-5 text-lg leading-relaxed text-muted">
        {doc.intro.map((paragraph) => (
          <p key={paragraph.slice(0, 32)}>{paragraph}</p>
        ))}
      </div>

      <Section id="compare-table" title={locale === "en" ? "Side by side" : "Lado a lado"}>
        <div className="mt-5 rounded-xl border border-border">
          {/* Phones get one stacked block per row; the table only appears from md up. */}
          <table className="block w-full border-collapse text-left text-sm md:table">
            <thead className="hidden md:table-header-group">
              <tr className="border-b border-border bg-surface">
                <th scope="col" className="px-4 py-3 align-bottom font-medium text-muted">
                  {labels.tableFeature}
                </th>
                <th scope="col" className="px-4 py-3 align-bottom font-semibold text-foreground">
                  CloakView
                </th>
                <th scope="col" className="px-4 py-3 align-bottom font-semibold text-foreground">
                  {doc.name}
                </th>
              </tr>
            </thead>
            <tbody className="block md:table-row-group">
              {doc.rows.map((row) => (
                <tr key={row.label} className="block border-b border-border px-4 py-4 last:border-0 md:table-row md:px-0 md:py-0">
                  <th scope="row" className="block font-medium text-foreground md:table-cell md:px-4 md:py-4 md:align-top">
                    {row.label}
                  </th>
                  <td className="mt-2 block text-muted md:mt-0 md:table-cell md:px-4 md:py-4 md:align-top">
                    <span className="mb-1 block text-xs font-semibold uppercase tracking-wide text-foreground md:hidden">
                      CloakView
                    </span>
                    {row.we}
                  </td>
                  <td className="mt-3 block text-muted md:mt-0 md:table-cell md:px-4 md:py-4 md:align-top">
                    <span className="mb-1 block text-xs font-semibold uppercase tracking-wide text-foreground md:hidden">
                      {doc.name}
                    </span>
                    {row.they}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Section>

      <Section id="their-strengths" title={themHeading}>
        <ul className="mt-5 space-y-4">
          {doc.theyWin.map((point) => (
            <li key={point.title} className="rounded-xl border border-border bg-surface p-5">
              <p className="font-medium text-foreground">{point.title}</p>
              <p className="mt-1 text-muted">{point.body}</p>
            </li>
          ))}
        </ul>
      </Section>

      <Section id="our-strengths" title={labels.weWinHeading}>
        <ul className="mt-5 space-y-4">
          {doc.weWin.map((point) => (
            <li key={point.title} className="rounded-xl border border-border bg-surface p-5">
              <p className="font-medium text-foreground">{point.title}</p>
              <p className="mt-1 text-muted">{point.body}</p>
            </li>
          ))}
        </ul>
      </Section>

      <Section id="verdict" title={labels.verdictHeading}>
        <p className="mt-4 text-lg leading-relaxed text-muted">{doc.verdict}</p>
      </Section>

      <Section id="faq" title={labels.faqHeading}>
        <div className="mt-5 space-y-3">
          {doc.faq.map((item) => (
            <details key={item.q} className="rounded-xl border border-border bg-surface px-5 py-4">
              <summary className="cursor-pointer font-medium text-foreground">{item.q}</summary>
              <p className="mt-3 text-muted">{item.a}</p>
            </details>
          ))}
        </div>
      </Section>

      <section className="mt-16 rounded-2xl border border-border bg-surface p-8 text-center">
        <h2 className="font-display text-2xl font-semibold text-foreground">{labels.ctaTitle}</h2>
        <p className="mx-auto mt-3 max-w-2xl text-muted">{labels.ctaBody}</p>
        <Link
          href={locale === "en" ? "/download" : "/es/download"}
          className="mt-6 inline-flex items-center justify-center rounded-lg bg-accent px-6 py-3 font-medium text-background transition-opacity hover:opacity-90"
        >
          {labels.ctaButton}
        </Link>
        <p className="mt-4 text-sm text-muted">{labels.ctaNote}</p>
      </section>

      <Section id="others" title={labels.othersHeading}>
        <ul className="mt-5 grid gap-4 sm:grid-cols-2">
          {others.map((other) => (
            <li key={other}>
              <Link
                href={comparePath(locale, other)}
                className="block rounded-xl border border-border bg-surface p-5 hover:border-accent"
              >
                <span className="font-medium text-foreground">
                  CloakView vs {compareDocs[locale][other].name}
                </span>
                <span className="mt-1 block text-sm text-muted">{compareDocs[locale][other].standfirst}</span>
                <span className="mt-3 block text-sm text-accent">{labels.readLabel} →</span>
              </Link>
            </li>
          ))}
        </ul>
      </Section>
    </article>
  );
}
