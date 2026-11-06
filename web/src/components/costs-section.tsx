import type { Dictionary } from "@/lib/get-dictionary";

export function CostsSection({
  dict,
}: {
  dict: Dictionary["home"]["costs"];
}) {
  return (
    <section className="border-t border-border">
      <div className="mx-auto max-w-5xl px-6 py-20">
        <h2 className="text-3xl font-bold tracking-tight text-foreground">
          {dict.title}
        </h2>
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
