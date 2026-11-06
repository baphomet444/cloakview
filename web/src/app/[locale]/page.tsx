import type { Metadata } from "next";
import { getDictionary } from "@/lib/get-dictionary";
import { siteConfig, type Locale } from "@/site.config";

export async function generateMetadata({
  params,
}: {
  params: Promise<{ locale: Locale }>;
}): Promise<Metadata> {
  const { locale } = await params;
  const dict = await getDictionary(locale);
  return { title: dict.home.title };
}

export default async function HomePage({
  params,
}: {
  params: Promise<{ locale: Locale }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale);

  return (
    <div className="mx-auto max-w-5xl px-6 py-24">
      <h1 className="text-3xl font-semibold text-foreground">
        {dict.home.heading}
      </h1>
      <p className="mt-4 text-muted">{siteConfig.description[locale]}</p>
    </div>
  );
}
