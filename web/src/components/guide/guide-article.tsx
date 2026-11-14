import Link from "next/link";
import type { ReactNode } from "react";
import type { Dictionary } from "@/lib/get-dictionary";
import { guidePath, guideSlugs, type GuideSlug } from "@/lib/guide";
import { GuideToc } from "@/components/guide/guide-toc";
import { siteConfig, type Locale } from "@/site.config";
import { breadcrumbJsonLd } from "@/lib/schema-generator";

export function GuideArticle({ locale, slug, guide, sections, content }: {
  locale: Locale; slug: GuideSlug; guide: Dictionary["guide"];
  sections: { id: string; title: string }[]; content: ReactNode;
}) {
  const position = guideSlugs.indexOf(slug);
  const previous = guideSlugs[position - 1];
  const next = guideSlugs[position + 1];
  return <div className="mx-auto max-w-6xl px-6 py-14">
    <script
      type="application/ld+json"
      dangerouslySetInnerHTML={{ __html: breadcrumbJsonLd([
        { name: guide.homeLabel, url: locale === "en" ? siteConfig.url : `${siteConfig.url}/es` },
        { name: guide.title, url: `${siteConfig.url}${guidePath(locale)}` },
        { name: guide.articles[slug].title, url: `${siteConfig.url}${guidePath(locale, slug)}` },
      ]) }}
    />
    <nav aria-label={guide.breadcrumbLabel} className="mb-10 text-sm text-muted">
      <ol className="flex flex-wrap items-center gap-2">
        <li><Link href={locale === "en" ? "/" : "/es"} className="hover:text-accent">{guide.homeLabel}</Link></li>
        <li aria-hidden="true">/</li>
        <li><Link href={guidePath(locale)} className="hover:text-accent">{guide.title}</Link></li>
        <li aria-hidden="true">/</li>
        <li aria-current="page" className="text-foreground">{guide.articles[slug].title}</li>
      </ol>
    </nav>
    <div className="grid gap-12 lg:grid-cols-[minmax(0,1fr)_16rem]">
      <div className="min-w-0">
        <h1 className="font-display text-4xl font-bold tracking-tight sm:text-5xl">{guide.articles[slug].title}</h1>
        <p className="mt-5 text-lg text-muted">{guide.articles[slug].description}</p>
        <article className="mt-10 leading-7 text-foreground/90 [&_p]:my-4 [&_ul]:my-4 [&_ul]:list-disc [&_ul]:pl-6 [&_ol]:my-4 [&_ol]:list-decimal [&_ol]:pl-6 [&_li]:my-2 [&_strong]:text-foreground [&_code]:font-mono [&_code]:text-accent">{content}</article>
        <nav aria-label={guide.articleNavigationLabel} className="mt-14 flex justify-between gap-5 border-t border-border pt-6 text-sm">
          {previous ? <Link className="text-accent hover:underline" href={guidePath(locale, previous)}>{guide.previousLabel}: {guide.articles[previous].title}</Link> : <span />}
          {next ? <Link className="text-accent hover:underline" href={guidePath(locale, next)}>{guide.nextLabel}: {guide.articles[next].title}</Link> : <span />}
        </nav>
      </div>
      <GuideToc sections={sections} label={guide.onThisPageLabel} />
    </div>
  </div>;
}
