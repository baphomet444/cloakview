import type { Dictionary } from "@/lib/get-dictionary";

/**
 * Bento: las dos primeras features del diccionario son las grandes, las
 * cuatro siguientes las pequeñas (el orden vive en en.json/es.json).
 */
export function FeaturesGrid({
  dict,
}: {
  dict: Dictionary["home"]["features"];
}) {
  const [first, second, ...rest] = dict.items;
  const large = [first, second];

  return (
    <section>
      <div className="mx-auto max-w-7xl px-6 py-24 lg:py-32">
        <h2 className="max-w-2xl font-display text-4xl leading-[0.95] font-semibold tracking-[-0.02em] text-foreground sm:text-5xl">
          {dict.title}
        </h2>
        <p className="mt-4 text-lg text-muted">{dict.subtitle}</p>
        <ul className="mt-14 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          {large.map((item, i) => (
            <li
              key={item.title}
              className="elevated flex min-h-64 flex-col justify-end rounded-md p-8 sm:col-span-2 lg:min-h-80"
            >
              <span
                aria-hidden="true"
                className="mb-auto font-display text-sm text-accent italic"
              >
                {String(i + 1).padStart(2, "0")}
              </span>
              <h3 className="font-display text-3xl leading-[1] font-semibold tracking-[-0.01em] text-foreground sm:text-4xl">
                {item.title}
              </h3>
              <p className="mt-3 max-w-md text-lg leading-relaxed text-muted">
                {item.desc}
              </p>
            </li>
          ))}
          {rest.map((item) => (
            <li key={item.title} className="elevated rounded-md p-6">
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
