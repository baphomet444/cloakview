import Link from "next/link";
import type { Dictionary } from "@/lib/get-dictionary";
import { guidePath, guideSlugs, type GuideSlug } from "@/lib/guide";
import type { Locale } from "@/site.config";

export function GuideIndex({ locale, guide }: { locale: Locale; guide: Dictionary["guide"] }) {
  return <section className="mx-auto max-w-5xl px-6 py-20">
    <h1 className="font-display text-4xl font-bold tracking-tight sm:text-5xl">{guide.title}</h1>
    <p className="mt-5 max-w-2xl text-lg text-muted">{guide.description}</p>
    <ul className="mt-12 grid gap-5 sm:grid-cols-2">
      {guideSlugs.map((slug: GuideSlug) => <li key={slug}>
        <Link href={guidePath(locale, slug)} className="block h-full rounded-xl border border-border bg-surface p-6 transition-colors hover:border-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent">
          <h2 className="font-display text-xl font-semibold text-foreground">{guide.articles[slug].title}</h2>
          <p className="mt-2 text-muted">{guide.articles[slug].description}</p>
        </Link>
      </li>)}
    </ul>
  </section>;
}
