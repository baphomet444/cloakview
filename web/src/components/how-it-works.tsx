import Image from "next/image";
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
        {/* La banda es el encabezado real de la ventana: muestra la barra de pestañas
            y el arranque de los paneles, sin texto que haya que leer de cerca. */}
        <figure className="mx-auto mt-12 w-full max-w-5xl">
          {/* En pantallas angostas el ancho completo encoge los rótulos a la nada,
              así que ahí se recorta a la barra de pestañas; los paneles asomando
              a la mitad se leen como un error de recorte. */}
          <Image
            src="/media/panels-band-crop.jpg"
            alt={dict.bandAlt}
            width={900}
            height={280}
            sizes="calc(100vw - 3rem)"
            className="h-auto w-full rounded-2xl border border-border shadow-[inset_0_1px_0_rgb(255_255_255/0.06),0_30px_60px_-30px_rgb(0_0_0/0.9)] sm:hidden"
          />
          <Image
            src="/media/panels-band.jpg"
            alt={dict.bandAlt}
            width={1280}
            height={426}
            sizes="(min-width: 1024px) 1024px, calc(100vw - 3rem)"
            className="hidden h-auto w-full rounded-2xl border border-border shadow-[inset_0_1px_0_rgb(255_255_255/0.06),0_30px_60px_-30px_rgb(0_0_0/0.9)] sm:block"
          />
        </figure>
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
