import type { Dictionary } from "@/lib/get-dictionary";

export function Faq({ dict }: { dict: Dictionary["home"]["faq"] }) {
  return (
    <section id="faq" className="border-t border-border">
      <div className="mx-auto max-w-3xl px-6 py-20">
        <h2 className="text-3xl font-bold tracking-tight text-foreground">
          {dict.title}
        </h2>
        <div className="mt-10 divide-y divide-border rounded-xl border border-border">
          {dict.items.map((item) => (
            <details key={item.q} className="group px-6 py-4">
              <summary className="flex cursor-pointer list-none items-center justify-between font-medium text-foreground [&::-webkit-details-marker]:hidden">
                {item.q}
                <span
                  aria-hidden="true"
                  className="ml-4 text-muted transition-transform group-open:rotate-45"
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
