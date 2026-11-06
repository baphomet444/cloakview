import type { Dictionary } from "@/lib/get-dictionary";

export function FeaturesGrid({
  dict,
}: {
  dict: Dictionary["home"]["features"];
}) {
  return (
    <section className="border-t border-border">
      <div className="mx-auto max-w-5xl px-6 py-20">
        <h2 className="text-3xl font-bold tracking-tight text-foreground">
          {dict.title}
        </h2>
        <p className="mt-3 text-lg text-muted">{dict.subtitle}</p>
        <ul className="mt-12 grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {dict.items.map((item) => (
            <li key={item.title} className="rounded-xl border border-border p-6">
              <h3 className="text-lg font-semibold text-foreground">
                {item.title}
              </h3>
              <p className="mt-2 leading-relaxed text-muted">{item.desc}</p>
            </li>
          ))}
        </ul>
      </div>
    </section>
  );
}
