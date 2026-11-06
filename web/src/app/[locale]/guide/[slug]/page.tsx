import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { GuideArticle } from "@/components/guide/guide-article";
import { getDictionary } from "@/lib/get-dictionary";
import { guidePath, guideSlugs, isGuideSlug } from "@/lib/guide";
import { loadArticle } from "@/lib/guide";
import { siteConfig, type Locale } from "@/site.config";

type Params = { params: Promise<{ locale: Locale; slug: string }> };
export function generateStaticParams() {
  return siteConfig.locales.flatMap((locale) => guideSlugs.map((slug) => ({ locale, slug })));
}
export async function generateMetadata({ params }: Params): Promise<Metadata> {
  const { locale, slug } = await params;
  if (!isGuideSlug(slug)) notFound();
  const { guide } = await getDictionary(locale);
  const article = guide.articles[slug];
  return {
    title: article.title, description: article.description,
    alternates: { canonical: guidePath(locale, slug), languages: { en: guidePath("en", slug), es: guidePath("es", slug) } },
    openGraph: { title: article.title, description: article.description, url: `${siteConfig.url}${guidePath(locale, slug)}`, locale: locale === "en" ? "en_US" : "es_ES" },
  };
}
export default async function GuideArticlePage({ params }: Params) {
  const { locale, slug } = await params;
  if (!isGuideSlug(slug)) notFound();
  const { guide } = await getDictionary(locale);
  const { sections, content } = await loadArticle(locale, slug, guide.copyLabel, guide.copiedLabel, guide.copyErrorLabel);
  return <GuideArticle locale={locale} slug={slug} guide={guide} sections={sections} content={content} />;
}
