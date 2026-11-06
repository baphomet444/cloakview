import type { Metadata } from "next";
import { getDictionary } from "@/lib/get-dictionary";
import type { Locale } from "@/site.config";
import { PricingHero } from "@/components/pricing/pricing-hero";
import { CostEstimator } from "@/components/pricing/cost-estimator";
import { RoadmapNote } from "@/components/pricing/roadmap-note";
import { DownloadButton } from "@/components/download-button";

export async function generateMetadata({
  params,
}: {
  params: Promise<{ locale: Locale }>;
}): Promise<Metadata> {
  const { locale } = await params;
  const dict = await getDictionary(locale);
  return {
    title: dict.pricing.title,
    description: dict.pricing.description,
  };
}

export default async function PricingPage({
  params,
}: {
  params: Promise<{ locale: Locale }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale);

  return (
    <>
      <PricingHero dict={dict.pricing.hero} />
      <CostEstimator dict={dict.pricing.estimator} />
      <RoadmapNote dict={dict.pricing.roadmap} />
      <section className="border-t border-border">
        <div className="mx-auto max-w-5xl px-6 py-20 text-center">
          <h2 className="text-3xl font-bold tracking-tight text-foreground">
            {dict.pricing.cta.title}
          </h2>
          <p className="mx-auto mt-4 max-w-2xl text-lg leading-relaxed text-muted">
            {dict.pricing.cta.body}
          </p>
          <div className="mt-8">
            <DownloadButton
              label={dict.pricing.cta.button}
              soonLabel={dict.pricing.cta.soon}
            />
          </div>
          <p className="mt-6 text-sm text-muted">{dict.pricing.cta.note}</p>
        </div>
      </section>
    </>
  );
}
