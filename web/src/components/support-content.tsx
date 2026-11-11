import { siteConfig, isGithubPending, type Locale } from "@/site.config";
import type { Dictionary } from "@/lib/get-dictionary";
import { guidePath } from "@/lib/guide";
import Link from "next/link";

export function SupportContent({ support, locale }: { support: Dictionary["support"]; locale: Locale }) {
  return <div className="grid gap-8 md:grid-cols-2">
    <section className="rounded-xl border border-border bg-surface p-6">
      <h2 className="font-display text-xl font-semibold">{support.emailTitle}</h2>
      <p className="mt-3 text-muted">{support.emailText} <a className="text-accent underline focus-visible:outline-2 focus-visible:outline-accent" href={`mailto:${siteConfig.SUPPORT_EMAIL}`}>{siteConfig.SUPPORT_EMAIL}</a></p>
    </section>
    <section className="rounded-xl border border-border bg-surface p-6">
      <h2 className="font-display text-xl font-semibold">{support.issuesTitle}</h2>
      <p className="mt-3 text-muted">{support.issuesText}</p>
      {isGithubPending() ? <p className="mt-3 text-muted" aria-disabled="true">{support.issuesPending}</p> : <a className="mt-3 inline-block text-accent underline focus-visible:outline-2 focus-visible:outline-accent" href={`${siteConfig.GITHUB_REPO}/issues`} rel="noopener noreferrer">{support.issuesLink}</a>}
    </section>
    <section className="md:col-span-2 rounded-xl border border-border bg-surface p-6">
      <h2 className="font-display text-xl font-semibold">{support.reportTitle}</h2>
      <ul className="mt-4 list-disc space-y-2 pl-6 text-muted">{support.include.map((item) => <li key={item}>{item}</li>)}</ul>
      <h3 className="mt-6 font-semibold">{support.excludeTitle}</h3>
      <p className="mt-2 text-muted">{support.exclude}</p>
      <Link className="mt-6 inline-block text-accent underline focus-visible:outline-2 focus-visible:outline-accent" href={guidePath(locale, "troubleshooting")}>{support.troubleshooting}</Link>
    </section>
  </div>;
}
