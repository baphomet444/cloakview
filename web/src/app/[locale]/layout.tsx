import { Geist, Geist_Mono } from "next/font/google";
import { notFound } from "next/navigation";
import type { Metadata } from "next";
import "./globals.css";
import { siteConfig, type Locale } from "@/site.config";
import { getDictionary } from "@/lib/get-dictionary";
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
      siteName: dict.meta.siteName,
      locale: locale === "en" ? "en_US" : "es_ES",
      description: siteConfig.description[locale],
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
      className={`${geistSans.variable} ${geistMono.variable} h-full`}
    >
      <body className="min-h-full antialiased flex flex-col">
        <NavBar dict={dict} locale={locale} />
        <main className="flex-1">{children}</main>
        <Footer dict={dict} locale={locale} />
      </body>
    </html>
  );
}
