import { siteConfig, type Locale } from "@/site.config";

export function generateSoftwareApplicationSchema(locale: Locale) {
  const isSpanish = locale === "es";

  return {
    "@context": "https://schema.org",
    "@type": "SoftwareApplication",
    name: "CloakView",
    description: siteConfig.description[locale],
    url: siteConfig.url,
    applicationCategory: "Productivity",
    operatingSystem: "macOS 14.2+",
    author: {
      "@type": "Organization",
      name: "CloakView",
      url: siteConfig.url,
    },
    offers: {
      "@type": "Offer",
      price: "0",
      priceCurrency: "USD",
      description: isSpanish
        ? "Gratis en v1. Sin suscripción, sin cuenta, sin intermediarios."
        : "Free in v1. No subscription, no account, no middleman.",
      availability: "https://schema.org/InStock",
      url: siteConfig.GITHUB_RELEASES,
    },
    downloadUrl: siteConfig.GITHUB_RELEASES,
    softwareRequirements: "macOS 14.2 or later (Apple Silicon or Intel)",
    featureList: [
      "Live transcription",
      "Private overlay",
      "Bring your own API key",
      "Local-first",
      "Works with Zoom, Teams, Google Meet",
    ],
  };
}
