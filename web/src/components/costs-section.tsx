import type { Dictionary } from "@/lib/get-dictionary";

/** Panel ancho único y elevado. */
export function CostsSection({
  dict,
}: {
  dict: Dictionary["home"]["costs"];
}) {
  return (
    <section>
      <div className="mx-auto max-w-7xl px-6 pb-24 lg:pb-32">
        <div className="elevated grid gap-10 rounded-md p-8 sm:p-12 lg:grid-cols-12 lg:gap-12 lg:p-16">
          <div className="lg:col-span-7">
            <h2 className="font-display text-4xl leading-[0.92] font-extrabold tracking-[-0.03em] text-foreground sm:text-5xl">
              {dict.title}
            </h2>
            <p className="mt-6 text-lg leading-relaxed text-muted">
              {dict.body}
            </p>
          </div>
          <ul className="space-y-5 self-end lg:col-span-5 lg:border-l lg:border-border lg:pl-12">
            {dict.points.map((point) => (
              <li key={point} className="flex items-start gap-3 text-foreground">
                <span
                  aria-hidden="true"
                  className="mt-2 inline-block h-1.5 w-1.5 shrink-0 rounded-full bg-accent shadow-[0_0_8px_rgb(255_107_26/0.7)]"
                />
                <span>{point}</span>
              </li>
            ))}
          </ul>
        </div>
      </div>
    </section>
  );
}
