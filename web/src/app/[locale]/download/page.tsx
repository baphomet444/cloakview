import type { Metadata } from "next";
import { Artifacts } from "@/components/download/artifacts";
import { MediaFigure } from "@/components/media-figure";
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
        <h1 className="font-display mt-6 max-w-3xl text-4xl font-bold tracking-tight text-foreground sm:text-5xl">{dict.download.heading}</h1>
        <p className="mt-6 max-w-3xl text-lg leading-relaxed text-muted">{dict.download.intro}</p>
      </section>
      <Artifacts dict={dict.download} />

      <section className="mx-auto max-w-5xl px-6 pt-2">
        <MediaFigure
          src="/media/app-window-toolbar.jpg"
          width={721}
          height={107}
          alt={dict.download.figure.alt}
          caption={dict.download.figure.caption}
          maxWidthClass="max-w-2xl"
        />
      </section>

      <section className="mx-auto max-w-5xl px-6 py-16">
        <div className="grid gap-12 md:grid-cols-2">
          <div>
            <h2 className="font-display text-2xl font-bold tracking-tight text-foreground">
              {dict.download.whatYouGet.title}
            </h2>
            <p className="mt-3 text-muted">{dict.download.whatYouGet.intro}</p>
            <ul className="mt-5 space-y-3">
              {dict.download.whatYouGet.items.map((item) => (
                <li key={item} className="flex gap-3 text-muted">
                  <span aria-hidden="true" className="mt-2 h-1.5 w-1.5 shrink-0 rounded-full bg-accent" />
                  <span>{item}</span>
                </li>
              ))}
            </ul>
          </div>
          <div>
            <h2 className="font-display text-2xl font-bold tracking-tight text-foreground">
              {dict.download.requirements.title}
            </h2>
            <ul className="mt-5 space-y-3">
              {dict.download.requirements.items.map((item) => (
                <li key={item} className="flex gap-3 text-muted">
                  <span aria-hidden="true" className="mt-2 h-1.5 w-1.5 shrink-0 rounded-full bg-accent" />
                  <span>{item}</span>
                </li>
              ))}
            </ul>
          </div>
        </div>
      </section>

      <section className="border-t border-border">
        <div className="mx-auto max-w-5xl px-6 py-16">
          <h2 className="font-display text-2xl font-bold tracking-tight text-foreground">
            {dict.download.install.title}
          </h2>
          <ol className="mt-6 grid gap-6 md:grid-cols-3">
            {dict.download.install.steps.map((step, index) => (
              <li key={step} className="rounded-xl border border-border bg-surface p-5">
                <span className="font-display text-2xl font-bold text-accent">{index + 1}</span>
                <p className="mt-2 text-muted">{step}</p>
              </li>
            ))}
          </ol>
        </div>
      </section>

      <section className="mx-auto max-w-5xl px-6 pb-20">
        <div className="rounded-2xl border border-border bg-surface p-6">
          <h2 className="font-display text-xl font-semibold text-foreground">
            {dict.download.verify.title}
          </h2>
          <p className="mt-3 text-muted">{dict.download.verify.text}</p>
          <pre className="mt-4 overflow-x-auto rounded-lg bg-background p-4 text-sm text-foreground">
            <code>{dict.download.verify.command}</code>
          </pre>
        </div>
      </section>
    </>
  );
}
