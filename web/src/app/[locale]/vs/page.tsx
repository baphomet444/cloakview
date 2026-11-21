import type { Metadata } from "next";
import { CompareIndex } from "@/components/compare/compare-index";
import { compareIndex, comparePath } from "@/content/compare";
import { getDictionary } from "@/lib/get-dictionary";
import { siteConfig, type Locale } from "@/site.config";
import { ogImages, ogLocale, twitterCard } from "@/lib/seo";
import { breadcrumbJsonLd, faqPageJsonLd } from "@/lib/schema-generator";

export async function generateMetadata({ params }: { params: Promise<{ locale: Locale }> }): Promise<Metadata> {
  const { locale } = await params;
  const copy = compareIndex[locale];
  const { meta } = await getDictionary(locale);
  const path = comparePath(locale);
  return {
    title: copy.title,
    description: copy.description,
    alternates: {
      canonical: path,
      languages: {
        en: comparePath("en"),
        es: comparePath("es"),
        "x-default": comparePath("en"),
      },
    },
    openGraph: {
      type: "website",
      title: copy.title,
      description: copy.description,
      url: `${siteConfig.url}${path}`,
      locale: ogLocale(locale),
      images: ogImages(locale, meta.ogAlt),
    },
    twitter: twitterCard(locale, meta.ogAlt),
  };
}

export default async function CompareHubPage({ params }: { params: Promise<{ locale: Locale }> }) {
  const { locale } = await params;
  const copy = compareIndex[locale];
  const url = `${siteConfig.url}${comparePath(locale)}`;

  return (
    <>
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{
          __html: breadcrumbJsonLd([
            { name: locale === "en" ? "Home" : "Inicio", url: locale === "en" ? siteConfig.url : `${siteConfig.url}/es` },
            { name: locale === "en" ? "Compare" : "Comparativas", url },
          ]),
        }}
      />
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: faqPageJsonLd([{ title: copy.h1, items: copy.faq }]) }}
      />
      <CompareIndex locale={locale} />
    </>
  );
}
