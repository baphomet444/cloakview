import { useEffect, useRef, useState } from "react";
import { t, type CatalogKey } from "../lib/i18n";
import { subscribeTranscriptDelta } from "../lib/ipc";
import { errorCopyKey } from "../hub/hubLogic";
import { WindowFrame } from "../windows/WindowFrame";
import { useSettingsBoot } from "../windows/useSettingsBoot";
import { appendTurn, type TranscriptLine } from "./transcriptLogic";

/**
 * The live transcript: every line as it is transcribed, labelled by who said it. The
 * interviewer's lines are the bright ones; the user's own (microphone) are dimmer.
 * The hub opens this window when listening starts, so it is there before the first line.
 */
export function TranscriptApp() {
  const { locale, bootErrorKey } = useSettingsBoot();
  const [lines, setLines] = useState<TranscriptLine[]>([]);
  const [errorKey, setErrorKey] = useState<CatalogKey | null>(null);
  const end = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (bootErrorKey !== null) setErrorKey(bootErrorKey);
  }, [bootErrorKey]);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    subscribeTranscriptDelta((turn) => setLines((current) => appendTurn(current, turn)))
      .then((stop) => {
        if (cancelled) stop();
        else unlisten = stop;
      })
      .catch((error: unknown) => {
        if (!cancelled) setErrorKey(errorCopyKey(error));
      });
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  // Follow the conversation: the newest line stays in view.
  useEffect(() => {
    end.current?.scrollIntoView({ block: "end" });
  }, [lines]);

  return (
    <WindowFrame titleKey="listen.title" locale={locale} errorKey={errorKey} onDismissError={() => setErrorKey(null)}>
      {lines.length === 0 ? (
        <p className="win-empty">{t("listen.empty")}</p>
      ) : (
        <div className="tr-list" aria-live="polite">
          {lines.map((line) => (
            <p key={line.id} className={line.mine ? "tr-line tr-mine" : "tr-line tr-them"}>
              <span className="tr-who">{t(line.mine ? "listen.you" : "listen.interviewer")}:</span> {line.text}
            </p>
          ))}
          <div ref={end} />
        </div>
      )}
    </WindowFrame>
  );
}
