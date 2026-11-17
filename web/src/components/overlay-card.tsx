import type { Dictionary } from "@/lib/get-dictionary";

type HeroProps = {
  dict: Dictionary["home"]["hero"];
};

/** Turno de la transcripción que se resalta (al que responde la sugerencia). */
const HIGHLIGHTED_TURN = 0;


/**
 * Tarjeta de UI del overlay de CloakView, en JSX (no SVG): barra de ventana,
 * transcripción con un turno resaltado, burbuja de sugerencia y pie con el
 * indicador "escuchando". Todo el texto sale del diccionario.
 */
export function OverlayCard({ dict }: HeroProps) {
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
