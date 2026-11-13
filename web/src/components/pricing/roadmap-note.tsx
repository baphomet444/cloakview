import type { Dictionary } from "@/lib/get-dictionary";
import { siteConfig } from "@/site.config";

export function RoadmapNote({
  dict,
}: {
  dict: Dictionary["pricing"]["roadmap"];
}) {
  const checkoutUrl = siteConfig.PRO_CHECKOUT_URL;

  return (
    <section className="border-t border-border">
      <div className="mx-auto max-w-5xl px-6 py-20">
        <h2 className="font-display text-3xl font-bold tracking-tight text-foreground">
          {dict.title}
        </h2>
        <div className="mt-8 grid gap-8 md:grid-cols-[auto_1fr] md:items-start">
          <div className="rounded-2xl border border-accent/40 bg-surface p-6 text-center md:min-w-52">
            <p className="font-display text-5xl font-extrabold tracking-tight text-foreground">
              {dict.price}
            </p>
            <p className="mt-2 text-sm text-muted">{dict.priceNote}</p>
            {checkoutUrl ? (
              <a
                href={checkoutUrl}
                rel="noopener noreferrer"
                className="btn btn-primary mt-5"
              >
                {dict.buy}
              </a>
            ) : (
              <span
                aria-disabled="true"
                className="mt-5 inline-flex cursor-not-allowed rounded-lg border border-border px-5 py-2.5 text-sm text-muted"
              >
                {dict.soon}
              </span>
            )}
          </div>
          <p className="max-w-3xl text-lg leading-relaxed text-muted">{dict.body}</p>
        </div>
      </div>
    </section>
  );
}
