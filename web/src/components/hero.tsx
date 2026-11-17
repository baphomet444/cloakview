import Image from "next/image";
import type { Dictionary } from "@/lib/get-dictionary";
import { DownloadButton } from "@/components/download-button";

type HeroProps = {
  dict: Dictionary["home"]["hero"];
};

/**
 * Subrayado a mano alzada detrás de la palabra manuscrita. El trazo se
 * deforma con feTurbulence + feDisplacementMap para que no parezca un
 * vector perfecto. SVG pequeño y local: no afecta al primer pintado.
 */
function HandUnderline() {
  return (
    <svg
      aria-hidden="true"
      viewBox="0 0 300 40"
      preserveAspectRatio="none"
      className="pointer-events-none absolute -bottom-[0.12em] left-[-6%] -z-10 h-[0.32em] w-[112%] overflow-visible"
    >
      <defs>
        <filter id="cv-hand-rough" x="-5%" y="-50%" width="110%" height="200%">
          <feTurbulence type="fractalNoise" baseFrequency="0.035 0.6" numOctaves="2" seed="7" result="noise" />
          <feDisplacementMap in="SourceGraphic" in2="noise" scale="7" xChannelSelector="R" yChannelSelector="G" />
        </filter>
      </defs>
      <path
        d="M6 26 C 70 12, 150 10, 294 18 M24 34 C 110 24, 200 22, 280 28"
        fill="none"
        stroke="var(--accent)"
        strokeOpacity="0.6"
        strokeWidth="7"
        strokeLinecap="round"
        filter="url(#cv-hand-rough)"
      />
    </svg>
  );
}

/**
 * La foto del escritorio: CloakView abierto sobre una llamada, en una habitación
 * a oscuras. Es la imagen más grande del primer pantallazo, así que se sirve con
 * prioridad y sin lazy: es el LCP de la página.
 *
 * 1280×720 es todo lo que hay, así que se limita a 896 px de ancho: más grande
 * se vería blanda en una pantalla retina.
 */
function HeroShot({ dict }: HeroProps) {
  return (
    <figure className="mx-auto w-full max-w-4xl">
      <Image
        src="/media/desk-hero.jpg"
        alt={dict.heroImageAlt}
        width={1280}
        height={720}
        priority
        sizes="(min-width: 896px) 896px, calc(100vw - 3rem)"
        className="h-auto w-full rounded-2xl border border-border shadow-[inset_0_1px_0_rgb(255_255_255/0.06),0_40px_80px_-24px_rgb(0_0_0/0.9)]"
      />
    </figure>
  );
}

export function Hero({ dict }: HeroProps) {
  return (
    <section className="relative isolate overflow-x-clip">
      {/* Rejilla de 1px cada 56px con máscara radial */}
      <div aria-hidden="true" className="hero-grid pointer-events-none absolute inset-0 -z-10" />

      <div className="mx-auto max-w-5xl px-6 pt-16 pb-20 text-center sm:pt-24 lg:pt-28 lg:pb-28">
        <p className="kicker justify-center">
          <span aria-hidden="true" className="kicker-dot" />
          {dict.kicker}
        </p>

        <h1 className="mt-7 font-display text-[2.5rem] min-[360px]:text-5xl leading-[0.92] font-extrabold tracking-[-0.03em] text-balance text-foreground sm:text-7xl md:text-8xl lg:text-9xl">
          {dict.titleLead}{" "}
          <span className="relative isolate inline-block -rotate-2 px-[0.04em] font-hand text-[1.08em] leading-[0.8] font-bold tracking-normal text-accent">
            {dict.titleHand}
            <HandUnderline />
          </span>{" "}
          {dict.titleTail}
        </h1>

        <p className="mx-auto mt-8 max-w-2xl text-lg leading-relaxed text-muted sm:text-xl">
          {dict.subtitle}
        </p>

        <div className="relative mx-auto mt-10 flex w-fit flex-wrap items-center justify-center gap-3">
          {/* Anotación manuscrita (escritorio): a la izquierda, apuntando al CTA */}
          <p
            aria-hidden="true"
            className="pointer-events-none absolute top-1/2 right-full mr-3 hidden -translate-y-1/2 items-center gap-1 font-hand text-2xl leading-none font-bold whitespace-nowrap text-accent lg:flex"
          >
            <span className="-rotate-6">{dict.annotation}</span>
          </p>
          <DownloadButton label={dict.ctaDownload} soonLabel={dict.ctaSoon} />
          <a href="#how-it-works" className="btn btn-secondary">
            {dict.ctaHow}
          </a>
        </div>

        {/* Anotación manuscrita (móvil/tablet): debajo, apuntando arriba */}
        <p className="mt-3 flex items-center justify-center gap-1 font-hand text-2xl leading-none font-bold text-accent lg:hidden">
          <span className="-rotate-3">{dict.annotation}</span>
        </p>

        <p className="mt-6 text-sm text-muted">{dict.platformsNote}</p>

        <div className="mt-16 sm:mt-20">
          <HeroShot dict={dict} />
        </div>
      </div>
    </section>
  );
}
