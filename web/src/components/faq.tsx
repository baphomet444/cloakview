import type { Dictionary } from "@/lib/get-dictionary";

/** FAQ estrecha (max-w-3xl). */
export function Faq({ dict }: { dict: Dictionary["home"]["faq"] }) {
  return (
    <section id="faq" className="border-t border-border">
      <div className="mx-auto max-w-3xl px-6 py-24 lg:py-28">
        <h2 className="font-display text-4xl leading-[0.95] font-semibold tracking-[-0.02em] text-foreground sm:text-5xl">
          {dict.title}
        </h2>
        <div className="mt-12 divide-y divide-border border-y border-border">
          {dict.items.map((item) => (
            <details key={item.q} className="group py-5">
              <summary className="flex cursor-pointer list-none items-center justify-between text-lg font-medium text-foreground [&::-webkit-details-marker]:hidden">
                {item.q}
                <span
                  aria-hidden="true"
                  className="ml-4 font-display text-2xl leading-none text-accent transition-transform group-open:rotate-45"
                >
                  +
                </span>
              </summary>
              <p className="mt-3 leading-relaxed text-muted">{item.a}</p>
            </details>
          ))}
        </div>
      </div>
    </section>
  );
}
