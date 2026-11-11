import { Caveat, Geist, Geist_Mono, Inter_Tight } from "next/font/google";
import { notFound } from "next/navigation";
import Script from "next/script";
import type { Metadata } from "next";
import "./globals.css";
import { siteConfig, type Locale } from "@/site.config";
import { getDictionary } from "@/lib/get-dictionary";
import { generateSoftwareApplicationSchema } from "@/lib/schema-generator";
import { NavBar } from "@/components/nav-bar";
import { Footer } from "@/components/footer";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

// Display: Inter Tight 800 (tracking -0.03em, leading 0.92). Un solo peso
// para no cargar el eje variable completo en la ruta crítica.
const interTight = Inter_Tight({
  variable: "--font-inter-tight",
  subsets: ["latin"],
  weight: ["800"],
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
      },
    },
    openGraph: {
      type: "website",
      url: locale === "en" ? siteConfig.url : `${siteConfig.url}/es`,
      siteName: dict.meta.siteName,
      locale: locale === "en" ? "en_US" : "es_ES",
      description: siteConfig.description[locale],
      images: [
        {
          url: `${siteConfig.url}/api/og?locale=${locale}`,
          width: 1200,
          height: 630,
          alt: dict.meta.siteName,
        },
      ],
    },
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
      className={`${geistSans.variable} ${geistMono.variable} ${interTight.variable} ${caveat.variable} h-full`}
    >
      <head>
        <Script
          id="software-application-schema"
          type="application/ld+json"
          dangerouslySetInnerHTML={{
            __html: JSON.stringify(generateSoftwareApplicationSchema(locale)),
          }}
        />
      </head>
      <body className="min-h-full antialiased flex flex-col">
        <NavBar dict={dict} locale={locale} />
        <main className="flex-1">{children}</main>
        <Footer dict={dict} locale={locale} />
      </body>
    </html>
  );
}
