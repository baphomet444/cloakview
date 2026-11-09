import type { Dictionary } from "@/lib/get-dictionary";
import { DownloadButton } from "@/components/download-button";

type HeroProps = {
  dict: Dictionary["home"]["hero"];
};

/** Envuelve texto a un ancho aproximado en caracteres (solo presentación SVG). */
function wrapText(text: string, maxChars: number): string[] {
  const words = text.split(" ");
  const lines: string[] = [];
  let current = "";
  for (const word of words) {
    const next = current ? `${current} ${word}` : word;
    if (next.length > maxChars && current) {
      lines.push(current);
      current = word;
    } else {
      current = next;
    }
  }
  if (current) lines.push(current);
  return lines;
}

/**
 * Mockup SVG ORIGINAL del overlay de CloakView sobre una videollamada:
 * ventana de llamada con dos participantes + panel del overlay con
 * transcripción en vivo y sugerencia de respuesta. Ilustración propia,
 * sin capturas ni imágenes de stock. El marco, la sombra proyectada, la
 * rotación y la luz de borde los pone el contenedor en <Hero>.
 */
function OverlayMockup({ dict }: HeroProps) {
  const suggestionLines = wrapText(dict.mockup.suggestionText, 40);
  const transcriptBars = [250, 205, 262, 170];

  return (
    <svg
      viewBox="0 0 640 400"
      role="img"
      aria-label={dict.mockupAlt}
      className="block h-auto w-full"
    >
      <defs>
        <filter id="cv-overlay-shadow" x="-10%" y="-10%" width="120%" height="140%">
          <feDropShadow dx="0" dy="14" stdDeviation="14" floodColor="#000" floodOpacity="0.6" />
          <feDropShadow dx="0" dy="0" stdDeviation="10" floodColor="#f2a03d" floodOpacity="0.18" />
        </filter>
      </defs>

      {/* Ventana de videollamada */}
      <rect width="640" height="400" className="fill-surface" />
      {/* Barra de título */}
      <rect width="640" height="38" className="fill-background" opacity="0.55" />
      <line x1="0" y1="38" x2="640" y2="38" className="stroke-border" strokeWidth="1" />
      <circle cx="22" cy="19" r="5.5" fill="#ff5f57" opacity="0.85" />
      <circle cx="40" cy="19" r="5.5" fill="#febc2e" opacity="0.85" />
      <circle cx="58" cy="19" r="5.5" fill="#28c840" opacity="0.85" />
      <text x="320" y="23.5" textAnchor="middle" className="fill-muted" fontSize="12" fontWeight="500">
        {dict.mockup.windowTitle}
      </text>

      {/* Participante 1 (tú) */}
      <rect x="20" y="54" width="290" height="206" rx="8" className="fill-background" />
      <circle cx="165" cy="140" r="36" className="fill-border" />
      <circle cx="165" cy="128" r="14" className="fill-muted" opacity="0.8" />
      <path d="M 141 166 q 24 -24 48 0 v 8 h -48 z" className="fill-muted" opacity="0.8" />
      <text x="34" y="246" className="fill-foreground" fontSize="13" fontWeight="600">
        {dict.mockup.participant1}
      </text>

      {/* Participante 2 (hablando): anillo ámbar de hablante activo */}
      <rect x="330" y="54" width="290" height="206" rx="8" className="fill-background stroke-accent" strokeWidth="1.5" strokeOpacity="0.7" />
      <circle cx="475" cy="140" r="36" className="fill-border" />
      <circle cx="475" cy="140" r="42" fill="none" className="stroke-accent" strokeWidth="1.5" opacity="0.45" />
      <circle cx="475" cy="128" r="14" className="fill-foreground" opacity="0.75" />
      <path d="M 451 166 q 24 -24 48 0 v 8 h -48 z" className="fill-foreground" opacity="0.75" />
      <circle cx="349" cy="242" r="4" className="fill-accent" />
      <text x="359" y="246" className="fill-foreground" fontSize="13" fontWeight="600">
        {dict.mockup.participant2} · {dict.mockup.speaking}
      </text>

      {/* Barra de controles de la llamada */}
      <circle cx="280" cy="300" r="11" className="fill-border" />
      <circle cx="320" cy="300" r="15" className="fill-border" />
      <circle cx="360" cy="300" r="11" className="fill-border" />

      {/* --- Overlay de CloakView (privado, solo visible para el usuario) --- */}
      <g filter="url(#cv-overlay-shadow)">
        <rect x="150" y="190" width="468" height="190" rx="6" className="fill-background" />
      </g>
      <rect x="150.5" y="190.5" width="467" height="189" rx="6" fill="none" className="stroke-accent" strokeOpacity="0.55" />
      {/* Luz de borde superior del overlay */}
      <line x1="158" y1="191.5" x2="610" y2="191.5" className="stroke-accent" strokeOpacity="0.9" strokeWidth="1" />

      {/* Cabecera del overlay */}
      <text x="168" y="217" className="fill-foreground" fontSize="14" fontWeight="700">
        {dict.mockup.overlayTitle}
      </text>
      <circle cx="476" cy="212.5" r="3.5" className="fill-accent" />
      <text x="486" y="216.5" className="fill-accent" fontSize="10.5" fontWeight="600">
        {dict.mockup.live}
      </text>

      {/* Líneas de transcripción (barras de texto en vivo) */}
      {transcriptBars.map((w, i) => (
        <rect key={i} x="168" y={232 + i * 13} width={w} height="6" rx="3" className="fill-muted" opacity={0.55 - i * 0.1} />
      ))}

      {/* Separador */}
      <line x1="168" y1="292" x2="600" y2="292" className="stroke-border" strokeWidth="1" />

      {/* Sugerencia de respuesta */}
      <text x="168" y="312" className="fill-accent" fontSize="11" fontWeight="600" letterSpacing="0.06em">
        {dict.mockup.suggestionLabel}
      </text>
      {suggestionLines.map((line, i) => (
        <text key={i} x="168" y={334 + i * 17} className="fill-foreground" fontSize="13">
          {line}
        </text>
      ))}
    </svg>
  );
}

export function Hero({ dict }: HeroProps) {
  return (
    <section className="relative overflow-x-clip">
      <div className="mx-auto grid max-w-7xl items-center gap-14 px-6 pt-14 pb-20 sm:pt-20 lg:grid-cols-12 lg:gap-8 lg:pt-24 lg:pb-28">
        <div className="lg:col-span-5">
          <p className="kicker">
            <span aria-hidden="true" className="kicker-dot" />
            {dict.kicker}
          </p>
          <h1 className="mt-6 font-display text-5xl leading-[0.95] tracking-[-0.025em] text-foreground sm:text-6xl xl:text-7xl">
            <span className="block font-semibold">{dict.titleLead}</span>
            <span className="mt-1 block font-light italic text-accent">
              {dict.titleEmphasis}
            </span>
          </h1>
          <p className="mt-7 max-w-md text-lg leading-relaxed text-muted">
            {dict.subtitle}
          </p>
          <div className="mt-9 flex flex-wrap items-center gap-3">
            <DownloadButton label={dict.ctaDownload} soonLabel={dict.ctaSoon} />
            <a href="#how-it-works" className="btn btn-secondary">
              {dict.ctaHow}
            </a>
          </div>
          <p className="mt-7 text-sm text-muted">{dict.platformsNote}</p>
        </div>

        {/* Columna de 7: el mockup sangra por el borde derecho de la
            ventana y queda recortado por overflow-x-clip de la sección. */}
        <div className="lg:col-span-7">
          <div className="w-[125%] origin-top-left rotate-[1.5deg] sm:w-[112%] lg:w-[calc(100%+max(0px,(100vw-80rem)/2)+10rem)]">
            <div className="relative overflow-hidden rounded-[10px] bg-surface shadow-[0_0_0_1px_rgb(255_214_170/0.1),0_2px_4px_rgb(0_0_0/0.5),0_40px_80px_-24px_rgb(0_0_0/0.85),0_0_120px_-30px_rgb(242_160_61/0.35)]">
              <OverlayMockup dict={dict} />
              {/* Luz de borde cálida del marco, por encima del SVG */}
              <div
                aria-hidden="true"
                className="pointer-events-none absolute inset-0 rounded-[10px] shadow-[inset_0_1px_0_rgb(255_214_170/0.22),inset_0_0_0_1px_rgb(255_214_170/0.06)]"
              />
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
