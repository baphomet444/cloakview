import { Caveat, Geist, Geist_Mono, Unbounded } from "next/font/google";
import { notFound } from "next/navigation";
import type { Metadata } from "next";
import "./globals.css";
import { siteConfig, type Locale } from "@/site.config";
import { getDictionary } from "@/lib/get-dictionary";
import { softwareApplicationJsonLd, organizationJsonLd } from "@/lib/schema-generator";
import { absoluteUrl, ogImages, ogLocale, twitterCard } from "@/lib/seo";
import { NavBar } from "@/components/nav-bar";
import { Footer } from "@/components/footer";
import { AssistantWidget } from "@/components/assistant/assistant-widget";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

// Display: Unbounded 700, la misma que la imagen para redes (opengraph-image.tsx),
// para todos los títulos (`font-display`). Un solo peso para no cargar el eje variable
// completo en la ruta crítica; next/font la descarga al compilar y la sirve el propio sitio.
const unbounded = Unbounded({
  variable: "--font-unbounded",
  subsets: ["latin"],
  weight: ["700"],
});

// Manuscrita: Caveat 700, solo para la palabra rotada y las anotaciones.
// Sin preload: no es el elemento LCP.
const caveat = Caveat({
  variable: "--font-caveat",
  subsets: ["latin"],
  weight: ["700"],
  preload: false,
});

export function generateStaticParams() {
  return siteConfig.locales.map((locale) => ({ locale }));
}

function isLocale(value: string): value is Locale {
  return (siteConfig.locales as readonly string[]).includes(value);
}

export async function generateMetadata({
  params,
}: {
  params: Promise<{ locale: string }>;
}): Promise<Metadata> {
  const { locale } = await params;
  if (!isLocale(locale)) return {};
  const dict = await getDictionary(locale);

  return {
    metadataBase: new URL(siteConfig.url),
    title: {
      default: dict.meta.siteName,
      template: `%s · ${dict.meta.siteName}`,
    },
    description: siteConfig.description[locale],
    alternates: {
      canonical: locale === "en" ? "/" : "/es",
      languages: {
        en: "/",
        es: "/es",
        "x-default": "/",
      },
    },
    openGraph: {
      type: "website",
      url: absoluteUrl(locale),
      siteName: dict.meta.siteName,
      locale: ogLocale(locale),
      description: siteConfig.description[locale],
      images: ogImages(locale, dict.meta.ogAlt),
    },
    twitter: twitterCard(locale, dict.meta.ogAlt),
  };
}

export default async function LocaleLayout({
  children,
  params,
}: {
  children: React.ReactNode;
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  if (!isLocale(locale)) notFound();

  const dict = await getDictionary(locale);

  return (
    <html
      lang={locale}
      className={`${geistSans.variable} ${geistMono.variable} ${unbounded.variable} ${caveat.variable} h-full`}
    >
      <head>
        {/* A plain tag, not next/script: it must be in the server HTML for crawlers. */}
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{ __html: softwareApplicationJsonLd(locale) }}
        />
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{ __html: organizationJsonLd(locale) }}
        />
      </head>
      <body className="min-h-full antialiased flex flex-col">
        <NavBar dict={dict} locale={locale} />
        <main className="flex-1">{children}</main>
        <Footer dict={dict} locale={locale} />
        <AssistantWidget dict={dict.assistant} locale={locale} />
      </body>
    </html>
  );
}
