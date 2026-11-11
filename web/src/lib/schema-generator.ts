import { siteConfig, type Locale } from "@/site.config";

/**
 * JSON-LD `SoftwareApplication` for the layout. Every claim matches what the site says
 * elsewhere (pricing, download and features copy).
 *
 * Availability and the download URL are added only once a release is published, which
 * is when `SHA256_ARM64` gets its value (the same switch that enables the download
 * button). Until then there is nothing to download, and claiming `InStock` would be false.
 */
export function generateSoftwareApplicationSchema(locale: Locale) {
  const isSpanish = locale === "es";
  const released = siteConfig.SHA256_ARM64 !== null;

  return {
    "@context": "https://schema.org",
    "@type": "SoftwareApplication",
    name: siteConfig.name,
    description: siteConfig.description[locale],
    url: locale === "en" ? siteConfig.url : `${siteConfig.url}/es`,
    inLanguage: locale,
    // One of the values Google accepts; "Productivity" is not.
    applicationCategory: "BusinessApplication",
    operatingSystem: "macOS",
    softwareRequirements: isSpanish
      ? "macOS 14.2 o posterior, Mac con Apple Silicon"
      : "macOS 14.2 or later, Mac with Apple Silicon",
    image: `${siteConfig.url}/brand/logo.png`,
    author: { "@type": "Organization", name: siteConfig.name, url: siteConfig.url },
    offers: {
      "@type": "Offer",
      price: "0",
      priceCurrency: "USD",
      description: isSpanish
        ? "Gratis en v1. Sin suscripción, sin cuenta, sin intermediarios."
        : "Free in v1. No subscription, no account, no middleman.",
      ...(released ? { availability: "https://schema.org/InStock", url: siteConfig.GITHUB_RELEASES } : {}),
    },
    ...(released ? { downloadUrl: siteConfig.GITHUB_RELEASES } : {}),
    featureList: isSpanish
      ? [
          "Transcripción en directo",
          "Overlay privado, invisible al compartir pantalla",
          "Tu propia clave de API",
          "Local-first",
          "Funciona junto a Zoom, Teams y Google Meet",
        ]
      : [
          "Live transcription",
          "Private overlay, invisible in screen sharing",
          "Bring your own API key",
          "Local-first",
          "Works alongside Zoom, Teams and Google Meet",
        ],
  };
}

/** The schema as the text of a `<script type="application/ld+json">`, with `<` escaped (Next.js JSON-LD guide). */
export function softwareApplicationJsonLd(locale: Locale): string {
  return JSON.stringify(generateSoftwareApplicationSchema(locale)).replace(/</g, "\\u003c");
}
