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
    author: { "@type": "Organization", name: siteConfig.name, url: siteConfig.url, sameAs: [siteConfig.INSTAGRAM_URL] },
    sameAs: [siteConfig.INSTAGRAM_URL],
    // The free app, plus CloakView Pro only once it can actually be bought: the Pro
    // offer appears with `PRO_CHECKOUT_URL`, the same switch as the buy button.
    offers: [
      {
        "@type": "Offer",
        name: "CloakView",
        price: "0",
        priceCurrency: "USD",
        description: isSpanish
          ? "App gratuita. Sin suscripción y sin cuenta; usa tu propia clave de OpenAI."
          : "Free app. No subscription, no account; uses your own OpenAI key.",
        ...(released ? { availability: "https://schema.org/InStock", url: siteConfig.GITHUB_RELEASES } : {}),
      },
      ...(siteConfig.PRO_CHECKOUT_URL
        ? [
            {
              "@type": "Offer",
              name: "CloakView Pro",
              price: "99",
              priceCurrency: "USD",
              description: isSpanish ? "Pago único, sin suscripción." : "One-time payment, no subscription.",
              availability: "https://schema.org/InStock",
              url: siteConfig.PRO_CHECKOUT_URL,
            },
          ]
        : []),
    ],
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

/**
 * `Organization` + `WebSite`, site-wide. This is what brand panels read: one
 * entity with a logo, the support address and the social profiles it owns.
 */
export function generateOrganizationSchema(locale: Locale) {
  return {
    "@context": "https://schema.org",
    "@graph": [
      {
        "@type": "Organization",
        "@id": `${siteConfig.url}/#organization`,
        name: siteConfig.name,
        url: siteConfig.url,
        logo: `${siteConfig.url}/brand/logo.png`,
        email: siteConfig.SUPPORT_EMAIL,
        sameAs: [siteConfig.INSTAGRAM_URL, siteConfig.GITHUB_REPO],
      },
      {
        "@type": "WebSite",
        "@id": `${siteConfig.url}/#website`,
        name: siteConfig.name,
        url: siteConfig.url,
        inLanguage: locale,
        publisher: { "@id": `${siteConfig.url}/#organization` },
      },
    ],
  };
}

export function organizationJsonLd(locale: Locale): string {
  return JSON.stringify(generateOrganizationSchema(locale)).replace(/</g, "\\u003c");
}

/** `FAQPage` built from the questions the page actually renders. */
export function faqPageJsonLd(topics: { title: string; items: { q: string; a: string }[] }[]): string {
  return JSON.stringify({
    "@context": "https://schema.org",
    "@type": "FAQPage",
    mainEntity: topics.flatMap((topic) =>
      topic.items.map((item) => ({
        "@type": "Question",
        name: item.q,
        acceptedAnswer: { "@type": "Answer", text: item.a },
      })),
    ),
  }).replace(/</g, "\\u003c");
}

/** `BreadcrumbList` matching the breadcrumb the guide article renders. */
export function breadcrumbJsonLd(items: { name: string; url: string }[]): string {
  return JSON.stringify({
    "@context": "https://schema.org",
    "@type": "BreadcrumbList",
    itemListElement: items.map((item, index) => ({
      "@type": "ListItem",
      position: index + 1,
      name: item.name,
      item: item.url,
    })),
  }).replace(/</g, "\\u003c");
}
