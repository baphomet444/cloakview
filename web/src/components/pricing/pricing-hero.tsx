import type { Dictionary } from "@/lib/get-dictionary";

export function PricingHero({
  dict,
}: {
  dict: Dictionary["pricing"]["hero"];
}) {
  return (
    <section>
      <div className="mx-auto max-w-5xl px-6 py-20">
        <p className="inline-block rounded-full border border-border px-3 py-1 text-xs font-medium uppercase tracking-wide text-muted">
          {dict.badge}
        </p>
        <h1 className="font-display mt-6 max-w-3xl text-4xl font-bold tracking-tight text-foreground sm:text-5xl">
          {dict.title}
        </h1>
        <p className="mt-6 max-w-3xl text-lg leading-relaxed text-muted">
          {dict.body}
        </p>
        <ul className="mt-8 space-y-3">
          {dict.points.map((point) => (
            <li key={point} className="flex items-start gap-3 text-foreground">
              <span
                aria-hidden="true"
                className="mt-1.5 inline-block h-2 w-2 shrink-0 rounded-full bg-accent"
              />
              <span>{point}</span>
            </li>
          ))}
        </ul>
      </div>
    </section>
  );
}
