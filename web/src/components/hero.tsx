import type { Dictionary } from "@/lib/get-dictionary";
import { DownloadButton } from "@/components/download-button";

type HeroProps = {
  dict: Dictionary["home"]["hero"];
};

/** Turno de la transcripción que se resalta (al que responde la sugerencia). */
const HIGHLIGHTED_TURN = 0;

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
 * Tarjeta de UI del overlay de CloakView, en JSX (no SVG): barra de ventana,
 * transcripción con un turno resaltado, burbuja de sugerencia y pie con el
 * indicador "escuchando". Todo el texto sale del diccionario.
 */
function OverlayCard({ dict }: HeroProps) {
  const m = dict.mockup;
  return (
    <figure
      aria-label={dict.mockupAlt}
      className="mx-auto w-full max-w-3xl overflow-hidden rounded-2xl border border-border bg-surface text-left shadow-[inset_0_1px_0_rgb(255_255_255/0.06),0_40px_80px_-24px_rgb(0_0_0/0.9),0_0_120px_-40px_rgb(255_107_26/0.35)]"
    >
      {/* Barra de ventana */}
      <div className="flex items-center gap-3 border-b border-border px-4 py-3">
        <span aria-hidden="true" className="flex gap-1.5">
          <span className="h-3 w-3 rounded-full bg-[#ff5f57]" />
          <span className="h-3 w-3 rounded-full bg-[#febc2e]" />
          <span className="h-3 w-3 rounded-full bg-[#28c840]" />
        </span>
        <span className="min-w-0 flex-1 truncate text-center text-xs font-medium text-muted">
          {m.windowTitle}
        </span>
        <span className="flex items-center gap-1.5 text-xs font-semibold text-accent">
          <span aria-hidden="true" className="h-1.5 w-1.5 rounded-full bg-accent" />
          {m.live}
        </span>
      </div>

      {/* Transcripción */}
      <ol className="space-y-1 px-3 pt-4 sm:px-5">
        {m.turns.map((turn, i) => {
          const hl = i === HIGHLIGHTED_TURN;
          return (
            <li
              key={i}
              className={`rounded-lg border-l-2 px-3 py-2 text-sm leading-relaxed sm:text-[0.95rem] ${
                hl ? "border-accent bg-accent/10" : "border-transparent"
              }`}
            >
              <span className={`mr-2 font-semibold ${hl ? "text-accent" : "text-foreground"}`}>
                {turn.speaker}
              </span>
              <span className={hl ? "text-foreground" : "text-muted"}>{turn.text}</span>
            </li>
          );
        })}
      </ol>

      {/* Burbuja de sugerencia */}
      <div className="px-3 pt-4 pb-5 sm:px-5">
        <div className="ml-auto max-w-[34rem] rounded-2xl rounded-br-sm border border-accent/40 bg-background px-4 py-3">
          <p className="text-xs font-semibold tracking-[0.06em] text-accent uppercase">
            {m.suggestionLabel}
          </p>
          <p className="mt-1.5 text-sm leading-relaxed text-foreground sm:text-[0.95rem]">
            {m.suggestionText}
          </p>
        </div>
      </div>

      {/* Pie: indicador de escucha */}
      <div className="flex items-center justify-between gap-3 border-t border-border px-4 py-3 text-xs text-muted sm:px-5">
        <span className="flex items-center gap-2">
          <span aria-hidden="true" className="flex h-3.5 items-end gap-[3px]">
            {[0, 1, 2, 3].map((i) => (
              <span
                key={i}
                className="listen-bar block h-full w-[3px] rounded-full bg-accent"
                style={{ animationDelay: `${i * 140}ms` }}
              />
            ))}
          </span>
          {m.listening}
        </span>
        <span>{m.privateNote}</span>
      </div>
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

        <h1 className="mt-7 font-display text-5xl leading-[0.92] font-extrabold tracking-[-0.03em] text-balance text-foreground sm:text-7xl md:text-8xl lg:text-9xl">
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
          <OverlayCard dict={dict} />
        </div>
      </div>
    </section>
  );
}
