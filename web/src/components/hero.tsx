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
 * sin capturas ni imágenes de stock.
 */
function OverlayMockup({ dict }: HeroProps) {
  const suggestionLines = wrapText(dict.mockup.suggestionText, 36);
  const transcriptBars = [220, 180, 235, 150];

  return (
    <svg
      viewBox="0 0 640 400"
      role="img"
      aria-label={dict.mockupAlt}
      className="h-auto w-full"
    >
      {/* Ventana de videollamada */}
      <rect
        x="8"
        y="8"
        width="624"
        height="384"
        rx="14"
        className="fill-surface stroke-border"
        strokeWidth="1.5"
      />
      {/* Barra superior */}
      <circle cx="32" cy="32" r="5" className="fill-border" />
      <circle cx="50" cy="32" r="5" className="fill-border" />
      <circle cx="68" cy="32" r="5" className="fill-border" />
      <text
        x="320"
        y="36"
        textAnchor="middle"
        className="fill-muted"
        fontSize="12"
      >
        {dict.mockup.windowTitle}
      </text>

      {/* Participante 1 (tú) */}
      <rect
        x="28"
        y="56"
        width="270"
        height="200"
        rx="10"
        className="fill-background stroke-border"
        strokeWidth="1"
      />
      <circle cx="163" cy="140" r="34" className="fill-border" />
      <circle cx="163" cy="128" r="14" className="fill-muted" />
      <path
        d="M 139 164 q 24 -22 48 0 v 8 h -48 z"
        className="fill-muted"
      />
      <text
        x="44"
        y="244"
        className="fill-foreground"
        fontSize="13"
        fontWeight="600"
      >
        {dict.mockup.participant1}
      </text>

      {/* Participante 2 (hablando) */}
      <rect
        x="342"
        y="56"
        width="270"
        height="200"
        rx="10"
        className="fill-background stroke-border"
        strokeWidth="1"
      />
      <circle cx="477" cy="140" r="34" className="fill-border" />
      <circle cx="477" cy="128" r="14" className="fill-accent" opacity="0.7" />
      <path
        d="M 453 164 q 24 -22 48 0 v 8 h -48 z"
        className="fill-accent"
        opacity="0.7"
      />
      <circle cx="360" cy="244" r="4" className="fill-accent" />
      <text
        x="370"
        y="248"
        className="fill-foreground"
        fontSize="13"
        fontWeight="600"
      >
        {dict.mockup.participant2} · {dict.mockup.speaking}
      </text>

      {/* Barra de controles de la llamada */}
      <circle cx="320" cy="300" r="14" className="fill-border" />
      <circle cx="280" cy="300" r="10" className="fill-border" />
      <circle cx="360" cy="300" r="10" className="fill-border" />

      {/* --- Overlay de CloakView (privado, solo visible para el usuario) --- */}
      <rect
        x="150"
        y="196"
        width="460"
        height="176"
        rx="12"
        className="fill-background stroke-accent"
        strokeWidth="1.5"
        opacity="0.98"
      />
      {/* Cabecera del overlay */}
      <text
        x="168"
        y="222"
        className="fill-foreground"
        fontSize="14"
        fontWeight="700"
      >
        {dict.mockup.overlayTitle}
      </text>
      <rect x="520" y="206" width="74" height="20" rx="10" className="fill-accent" />
      <circle cx="532" cy="216" r="3" className="fill-accent-foreground" />
      <text
        x="540"
        y="220"
        className="fill-accent-foreground"
        fontSize="10"
        fontWeight="600"
      >
        {dict.mockup.live}
      </text>

      {/* Líneas de transcripción (barras de texto en vivo) */}
      {transcriptBars.map((w, i) => (
        <rect
          key={i}
          x="168"
          y={236 + i * 14}
          width={w}
          height="7"
          rx="3.5"
          className="fill-muted"
          opacity={0.5 - i * 0.08}
        />
      ))}

      {/* Separador */}
      <line
        x1="168"
        y1="296"
        x2="592"
        y2="296"
        className="stroke-border"
        strokeWidth="1"
      />

      {/* Sugerencia de respuesta */}
      <text
        x="168"
        y="316"
        className="fill-accent"
        fontSize="11"
        fontWeight="600"
      >
        {dict.mockup.suggestionLabel}
      </text>
      {suggestionLines.map((line, i) => (
        <text
          key={i}
          x="168"
          y={336 + i * 16}
          className="fill-foreground"
          fontSize="12.5"
        >
          {line}
        </text>
      ))}
    </svg>
  );
}

export function Hero({ dict }: HeroProps) {
  return (
    <section className="mx-auto max-w-5xl px-6 pt-16 pb-20 sm:pt-24">
      <div className="grid items-center gap-12 lg:grid-cols-2">
        <div>
          <p className="mb-4 inline-block rounded-full border border-border px-3 py-1 text-xs font-medium tracking-wide text-muted uppercase">
            {dict.badge}
          </p>
          <h1 className="text-4xl font-bold tracking-tight text-foreground sm:text-5xl">
            {dict.title}
          </h1>
          <p className="mt-6 max-w-xl text-lg leading-relaxed text-muted">
            {dict.subtitle}
          </p>
          <div className="mt-8 flex flex-wrap items-center gap-4">
            <DownloadButton label={dict.ctaDownload} soonLabel={dict.ctaSoon} />
            <a
              href="#how-it-works"
              className="rounded-lg border border-border px-5 py-2.5 font-medium text-foreground transition-colors hover:border-accent"
            >
              {dict.ctaHow}
            </a>
          </div>
          <p className="mt-6 text-sm text-muted">{dict.platformsNote}</p>
        </div>
        <OverlayMockup dict={dict} />
      </div>
    </section>
  );
}
