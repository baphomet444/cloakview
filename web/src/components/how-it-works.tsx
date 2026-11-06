import type { Dictionary } from "@/lib/get-dictionary";

export function HowItWorks({ dict }: { dict: Dictionary["home"]["how"] }) {
  return (
    <section id="how-it-works" className="border-t border-border">
      <div className="mx-auto max-w-5xl px-6 py-20">
        <h2 className="text-3xl font-bold tracking-tight text-foreground">
          {dict.title}
        </h2>
        <p className="mt-3 text-lg text-muted">{dict.subtitle}</p>
        <ol className="mt-12 grid gap-8 sm:grid-cols-3">
          {dict.steps.map((step, i) => (
            <li key={step.title} className="rounded-xl border border-border p-6">
              <span
                aria-hidden="true"
                className="inline-flex h-8 w-8 items-center justify-center rounded-full bg-accent font-mono text-sm font-bold text-accent-foreground"
              >
                {i + 1}
              </span>
              <h3 className="mt-4 text-lg font-semibold text-foreground">
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
