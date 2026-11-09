import type { Dictionary } from "@/lib/get-dictionary";

/** Banda horizontal a sangre con tres pasos numerados. */
export function HowItWorks({ dict }: { dict: Dictionary["home"]["how"] }) {
  return (
    <section
      id="how-it-works"
      className="scroll-mt-8 border-y border-border bg-surface shadow-[inset_0_1px_0_rgb(255_255_255/0.06)]"
    >
      <div className="mx-auto max-w-7xl px-6 py-16 lg:py-20">
        <div className="flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
          <h2 className="font-display text-4xl leading-[0.92] font-extrabold tracking-[-0.03em] text-foreground sm:text-5xl">
            {dict.title}
          </h2>
          <p className="max-w-sm text-lg text-muted lg:text-right">
            {dict.subtitle}
          </p>
        </div>
        <ol className="mt-12 grid border-t border-border md:grid-cols-3">
          {dict.steps.map((step, i) => (
            <li
              key={step.title}
              className="border-b border-border py-8 md:border-b-0 md:py-10 md:pr-10 md:not-first:border-l md:not-first:pl-10"
            >
              <span
                aria-hidden="true"
                className="font-display text-6xl leading-none font-extrabold text-accent"
              >
                {String(i + 1).padStart(2, "0")}
              </span>
              <h3 className="mt-6 text-xl font-semibold text-foreground">
                {step.title}
              </h3>
              <p className="mt-2 leading-relaxed text-muted">{step.desc}</p>
            </li>
          ))}
        </ol>
      </div>
    </section>
  );
}
