import type { Metadata } from "next";
import { getDictionary } from "@/lib/get-dictionary";
import { siteConfig, type Locale } from "@/site.config";
import { SupportContent } from "@/components/support-content";
import { ogImages, twitterCard } from "@/lib/seo";

const pathFor = (locale: Locale) => `${locale === "en" ? "" : "/es"}/support`;

export async function generateMetadata({ params }: { params: Promise<{ locale: Locale }> }): Promise<Metadata> {
  const { locale } = await params;
  const { support, meta } = await getDictionary(locale);
  return {
    title: support.title, description: support.description,
    alternates: { canonical: pathFor(locale), languages: { en: pathFor("en"), es: pathFor("es"), "x-default": pathFor("en") } },
    openGraph: { title: support.title, description: support.description, url: `${siteConfig.url}${pathFor(locale)}`, locale: locale === "en" ? "en_US" : "es_ES", images: ogImages(locale, meta.ogAlt) },
    twitter: twitterCard(locale, meta.ogAlt),
  };
}

export default async function Page({ params }: { params: Promise<{ locale: Locale }> }) {
  const { locale } = await params;
  const { support } = await getDictionary(locale);
  return <div className="mx-auto max-w-4xl px-6 py-16">
    <h1 className="text-4xl font-semibold tracking-tight">{support.title}</h1>
    <p className="mt-4 mb-12 text-lg text-muted">{support.intro}</p>
    <SupportContent support={support} locale={locale} />
  </div>;
}
