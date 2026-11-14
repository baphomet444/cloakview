import type { Metadata } from "next";
import { getDictionary } from "@/lib/get-dictionary";
import { siteConfig, type Locale } from "@/site.config";
import { FaqTopics } from "@/components/faq-topics";
import { ogImages, twitterCard } from "@/lib/seo";
import { faqPageJsonLd } from "@/lib/schema-generator";

const pathFor = (locale: Locale) => `${locale === "en" ? "" : "/es"}/faq`;

export async function generateMetadata({ params }: { params: Promise<{ locale: Locale }> }): Promise<Metadata> {
  const { locale } = await params;
  const { faq, meta } = await getDictionary(locale);
  return {
    title: faq.title, description: faq.description,
    alternates: { canonical: pathFor(locale), languages: { en: pathFor("en"), es: pathFor("es"), "x-default": pathFor("en") } },
    openGraph: { title: faq.title, description: faq.description, url: `${siteConfig.url}${pathFor(locale)}`, locale: locale === "en" ? "en_US" : "es_ES", images: ogImages(locale, meta.ogAlt) },
    twitter: twitterCard(locale, meta.ogAlt),
  };
}

export default async function Page({ params }: { params: Promise<{ locale: Locale }> }) {
  const { locale } = await params;
  const { faq } = await getDictionary(locale);
  return <div className="mx-auto max-w-4xl px-6 py-16">
    <script
      type="application/ld+json"
      dangerouslySetInnerHTML={{ __html: faqPageJsonLd(faq.topics) }}
    />
    <h1 className="font-display text-4xl font-semibold tracking-tight">{faq.title}</h1>
    <p className="mt-4 mb-12 text-lg text-muted">{faq.intro}</p>
    <FaqTopics faq={faq} locale={locale} />
  </div>;
}
