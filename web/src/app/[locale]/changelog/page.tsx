import type { Metadata } from "next";
import { getDictionary } from "@/lib/get-dictionary";
import { siteConfig, type Locale } from "@/site.config";
import { ChangelogContent } from "@/components/changelog-content";

const pathFor = (locale: Locale) => `${locale === "en" ? "" : "/es"}/changelog`;

export async function generateMetadata({ params }: { params: Promise<{ locale: Locale }> }): Promise<Metadata> {
  const { locale } = await params;
  const { changelog } = await getDictionary(locale);
  return {
    title: changelog.title, description: changelog.description,
    alternates: { canonical: pathFor(locale), languages: { en: pathFor("en"), es: pathFor("es") } },
    openGraph: { title: changelog.title, description: changelog.description, url: `${siteConfig.url}${pathFor(locale)}`, locale: locale === "en" ? "en_US" : "es_ES" },
  };
}

export default async function Page({ params }: { params: Promise<{ locale: Locale }> }) {
  const { locale } = await params;
  const { changelog } = await getDictionary(locale);
  return <div className="mx-auto max-w-4xl px-6 py-16">
    <h1 className="text-4xl font-semibold tracking-tight">{changelog.title}</h1>
    <p className="mt-4 mb-12 text-lg text-muted">{changelog.intro}</p>
    <ChangelogContent copy={changelog} locale={locale} />
  </div>;
}
