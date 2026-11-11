import { useEffect, useReducer, useRef, useState } from "react";
import { t, type CatalogKey } from "../lib/i18n";
import {
  requestAnswer,
  stopAnswer,
  subscribeAnswerDelta,
  subscribeAnswerDone,
  subscribeError,
  subscribeTranscriptDelta,
  subscribeTurnDetected,
} from "../lib/ipc";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { errorCopyKey } from "../hub/hubLogic";
import { WindowFrame } from "../windows/WindowFrame";
import { useSettingsBoot } from "../windows/useSettingsBoot";
import { ASK_IDLE, askReducer, isBusy, questionArg } from "./askLogic";

/**
 * Ask a question about the conversation (or send nothing, for a suggested reply) and
 * watch the answer stream in. The answer arrives over `answer-delta` / `answer-done`,
 * not the `request_answer` promise, which only says whether the request started.
 */
export function AskApp() {
  const { locale, bootErrorKey } = useSettingsBoot();
  const [state, dispatch] = useReducer(askReducer, ASK_IDLE);
  const [input, setInput] = useState("");
  // Failures outside a question (loading settings, subscribing): the reducer only
  // tracks errors of the question in flight.
  const [windowErrorKey, setWindowErrorKey] = useState<CatalogKey | null>(null);
  useEffect(() => {
    if (bootErrorKey !== null) setWindowErrorKey(bootErrorKey);
  }, [bootErrorKey]);
  const busy = isBusy(state);
  // Whether the question on screen was heard on the call rather than typed here.
  const [heard, setHeard] = useState(false);
  // The last thing said on the call: `turn-detected` carries no text, and always
  // follows the `transcript-delta` of the line that caused it.
  const lastHeard = useRef("");

  // The three answer events. Each `subscribe*` resolves to its unlisten function
  // asynchronously, so the cleanup copes with running before that happens.
  useEffect(() => {
    let cancelled = false;
    const stops: UnlistenFn[] = [];
    const keep = (stop: UnlistenFn) => (cancelled ? stop() : stops.push(stop));
    Promise.all([
      subscribeAnswerDelta((text) => dispatch({ type: "delta", text })).then(keep),
      subscribeAnswerDone(() => dispatch({ type: "done" })).then(keep),
      subscribeTranscriptDelta((turn) => {
        // The user's own words (microphone) are not what a suggestion answers.
        if (!turn.isUserTurn) lastHeard.current = turn.text;
      }).then(keep),
      // Something on the call asks for a reply: suggest one without being asked.
      // The transcript is the context, so no question is sent; a suggestion still
      // streaming is replaced (Rust aborts the previous request).
      subscribeTurnDetected(() => {
        setHeard(true);
        dispatch({ type: "send", question: lastHeard.current });
        requestAnswer().catch((error: unknown) => dispatch({ type: "error", key: errorCopyKey(error) }));
      }).then(keep),
      // Shared with the realtime side; the reducer ignores it unless a question is in flight.
      subscribeError(() => dispatch({ type: "error", key: "errors.generic" })).then(keep),
    ]).catch((error: unknown) => {
      if (!cancelled) setWindowErrorKey(errorCopyKey(error));
    });
    return () => {
      cancelled = true;
      stops.forEach((stop) => stop());
      // Closing the window must not leave an answer streaming to nobody.
      void stopAnswer().catch(() => undefined);
    };
  }, []);

  async function send(): Promise<void> {
    if (busy) return;
    const question = questionArg(input);
    setHeard(false);
    dispatch({ type: "send", question: question ?? "" });
    setInput("");
    try {
      await requestAnswer(question);
    } catch (error) {
      dispatch({ type: "error", key: errorCopyKey(error) });
    }
  }

  async function stop(): Promise<void> {
    dispatch({ type: "stop" });
    try {
      await stopAnswer();
    } catch (error) {
      dispatch({ type: "error", key: errorCopyKey(error) });
    }
  }

  const errorKey = state.errorKey ?? windowErrorKey;

  return (
    <WindowFrame
      titleKey="ask.title"
      locale={locale}
      errorKey={errorKey}
      onDismissError={() => {
        if (state.errorKey !== null) dispatch({ type: "dismiss" });
        else setWindowErrorKey(null);
      }}
      footer={
        <form
          className="win-row"
          onSubmit={(event) => {
            event.preventDefault();
            void send();
          }}
        >
          <input
            className="win-input"
            type="text"
            value={input}
            placeholder={t("ask.placeholder")}
            aria-label={t("ask.placeholder")}
            onChange={(event) => setInput(event.currentTarget.value)}
          />
          {busy ? (
            <button type="button" className="win-btn win-btn-primary" onClick={() => void stop()}>
              {t("ask.stop")}
            </button>
          ) : (
            <button type="submit" className="win-btn win-btn-primary">
              {t("ask.send")}
            </button>
          )}
        </form>
      }
    >
      {state.phase === "idle" ? (
        <p className="win-empty">{t("ask.empty")}</p>
      ) : (
        <div className="ask-thread" aria-live="polite">
          {state.question !== "" && (
            <section className="ask-block">
              <h2 className="win-label">{t(heard ? "ask.heard" : "ask.you")}</h2>
              <p className="ask-question">{state.question}</p>
            </section>
          )}
          <section className="ask-block">
            <h2 className="win-label">{t("ask.suggestion")}</h2>
            {state.phase === "thinking" ? (
              <p className="win-muted">{t("ask.thinking")}</p>
            ) : (
              <p className="ask-answer">{state.answer}</p>
            )}
          </section>
        </div>
      )}
    </WindowFrame>
  );
}
