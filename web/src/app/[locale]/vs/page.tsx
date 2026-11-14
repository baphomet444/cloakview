import type { Metadata } from "next";
import { CompareIndex } from "@/components/compare/compare-index";
import { compareIndex, comparePath } from "@/content/compare";
import { getDictionary } from "@/lib/get-dictionary";
import { siteConfig, type Locale } from "@/site.config";
import { ogImages, ogLocale, twitterCard } from "@/lib/seo";

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
  return <CompareIndex locale={locale} />;
}
