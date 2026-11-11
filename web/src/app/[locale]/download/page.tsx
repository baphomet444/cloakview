import type { Metadata } from "next";
import { Artifacts } from "@/components/download/artifacts";
import { getDictionary } from "@/lib/get-dictionary";
import { siteConfig, type Locale } from "@/site.config";
import { ogImages, twitterCard } from "@/lib/seo";

export async function generateMetadata({ params }: { params: Promise<{ locale: Locale }> }): Promise<Metadata> {
  const { locale } = await params;
  const dict = await getDictionary(locale);
  const path = locale === "en" ? "/download" : "/es/download";
  return {
    // The title already names the product: skip the layout's " · CloakView" suffix.
    title: { absolute: dict.download.title },
    description: dict.download.description,
    alternates: {
      canonical: path,
      languages: { en: "/download", es: "/es/download", "x-default": "/download" },
    },
    openGraph: {
      title: dict.download.title,
      description: dict.download.description,
      url: `${siteConfig.url}${path}`,
      locale: locale === "en" ? "en_US" : "es_ES",
      images: ogImages(locale, dict.meta.ogAlt),
    },
    twitter: twitterCard(locale, dict.meta.ogAlt),
  };
}

export default async function DownloadPage({ params }: { params: Promise<{ locale: Locale }> }) {
  const { locale } = await params;
  const dict = await getDictionary(locale);
  return (
    <>
      <section className="mx-auto max-w-5xl px-6 py-20">
        <p className="inline-block rounded-full border border-border px-3 py-1 text-xs font-medium uppercase tracking-wide text-muted">{dict.download.badge}</p>
        <h1 className="mt-6 max-w-3xl text-4xl font-bold tracking-tight text-foreground sm:text-5xl">{dict.download.heading}</h1>
        <p className="mt-6 max-w-3xl text-lg leading-relaxed text-muted">{dict.download.intro}</p>
      </section>
      <Artifacts dict={dict.download} />
    </>
  );
}
