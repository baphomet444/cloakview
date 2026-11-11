import type { Metadata } from "next";
import { GuideIndex } from "@/components/guide/guide-index";
import { getDictionary } from "@/lib/get-dictionary";
import { guidePath } from "@/lib/guide";
import { siteConfig, type Locale } from "@/site.config";
import { ogImages, twitterCard } from "@/lib/seo";

export async function generateMetadata({ params }: { params: Promise<{ locale: Locale }> }): Promise<Metadata> {
  const { locale } = await params;
  const { guide, meta } = await getDictionary(locale);
  return {
    title: guide.title, description: guide.description,
    alternates: { canonical: guidePath(locale), languages: { en: guidePath("en"), es: guidePath("es"), "x-default": guidePath("en") } },
    openGraph: { title: guide.title, description: guide.description, url: `${siteConfig.url}${guidePath(locale)}`, locale: locale === "en" ? "en_US" : "es_ES", images: ogImages(locale, meta.ogAlt) },
    twitter: twitterCard(locale, meta.ogAlt),
  };
}
export default async function GuidePage({ params }: { params: Promise<{ locale: Locale }> }) {
  const { locale } = await params;
  const { guide } = await getDictionary(locale);
  return <GuideIndex locale={locale} guide={guide} />;
}
