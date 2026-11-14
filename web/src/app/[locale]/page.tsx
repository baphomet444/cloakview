import type { Metadata } from "next";
import { getDictionary } from "@/lib/get-dictionary";
import type { Locale } from "@/site.config";
import { absoluteUrl, ogImages, ogLocale } from "@/lib/seo";
import { Hero } from "@/components/hero";
import { HowItWorks } from "@/components/how-it-works";
import { FeaturesGrid } from "@/components/features-grid";
import { CostsSection } from "@/components/costs-section";
import { Faq } from "@/components/faq";
import { DownloadCta } from "@/components/download-cta";

export async function generateMetadata({
  params,
}: {
  params: Promise<{ locale: Locale }>;
}): Promise<Metadata> {
  const { locale } = await params;
  const dict = await getDictionary(locale);
  return {
    title: dict.home.title,
    description: dict.home.description,
    openGraph: {
      type: "website",
      title: dict.home.title,
      description: dict.home.description,
      url: absoluteUrl(locale),
      locale: ogLocale(locale),
      images: ogImages(locale, dict.meta.ogAlt),
    },
  };
}

export default async function HomePage({
  params,
}: {
  params: Promise<{ locale: Locale }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale);

  return (
    <>
      <Hero dict={dict.home.hero} />
      <HowItWorks dict={dict.home.how} />
      <FeaturesGrid dict={dict.home.features} />
      <CostsSection dict={dict.home.costs} />
      <Faq dict={dict.home.faq} />
      <DownloadCta dict={dict.home.download} />
    </>
  );
}
