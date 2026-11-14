import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { CompareView } from "@/components/compare/compare-view";
import { compareDocs, comparePath, compareSlugs, isCompareSlug } from "@/content/compare";
import { getDictionary } from "@/lib/get-dictionary";
import { siteConfig, type Locale } from "@/site.config";
import { ogImages, ogLocale, twitterCard } from "@/lib/seo";
import { breadcrumbJsonLd, faqPageJsonLd } from "@/lib/schema-generator";

type Params = Promise<{ locale: Locale; slug: string }>;

export function generateStaticParams() {
  return compareSlugs.map((slug) => ({ slug }));
}

export async function generateMetadata({ params }: { params: Params }): Promise<Metadata> {
  const { locale, slug } = await params;
  if (!isCompareSlug(slug)) return {};
  const doc = compareDocs[locale][slug];
  const { meta } = await getDictionary(locale);
  const path = comparePath(locale, slug);
  return {
    title: doc.title,
    description: doc.description,
    alternates: {
      canonical: path,
      languages: {
        en: comparePath("en", slug),
        es: comparePath("es", slug),
        "x-default": comparePath("en", slug),
      },
    },
    openGraph: {
      type: "article",
      title: doc.title,
      description: doc.description,
      url: `${siteConfig.url}${path}`,
      locale: ogLocale(locale),
      images: ogImages(locale, meta.ogAlt),
    },
    twitter: twitterCard(locale, meta.ogAlt),
  };
}

export default async function ComparePage({ params }: { params: Params }) {
  const { locale, slug } = await params;
  if (!isCompareSlug(slug)) notFound();
  const doc = compareDocs[locale][slug];
  const url = `${siteConfig.url}${comparePath(locale, slug)}`;

  return (
    <>
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{
          __html: breadcrumbJsonLd([
            { name: locale === "en" ? "Home" : "Inicio", url: locale === "en" ? siteConfig.url : `${siteConfig.url}/es` },
            { name: locale === "en" ? "Compare" : "Comparativas", url: `${siteConfig.url}${comparePath(locale)}` },
            { name: doc.h1, url },
          ]),
        }}
      />
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: faqPageJsonLd([{ title: doc.h1, items: doc.faq }]) }}
      />
      <CompareView locale={locale} slug={slug} />
    </>
  );
}
