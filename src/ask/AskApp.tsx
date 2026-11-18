import { useEffect, useReducer, useRef, useState } from "react";
import { t, type CatalogKey } from "../lib/i18n";
import {
  announceAskReady,
  requestAnswer,
  stopAnswer,
  subscribeAnswerDelta,
  subscribeAnswerDone,
  subscribeAnswerPrompt,
  subscribeAnswerStarted,
  subscribeError,
  subscribeTranscriptDelta,
  subscribeTurnDetected,
} from "../lib/ipc";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { errorCopyKey } from "../hub/hubLogic";
import { RichText } from "../rich/RichText";
import { WindowFrame } from "../windows/WindowFrame";
import { useSettingsBoot } from "../windows/useSettingsBoot";
import { ASK_IDLE, askReducer, isBusy, isNearBottom, questionArg, shouldQueueHeard, type AskSource } from "./askLogic";

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
  // The last thing said on the call: `turn-detected` carries no text, and always
  // follows the `transcript-delta` of the line that caused it. Shown until Rust says
  // what the suggestion really answers (`answer-prompt`).
  const lastHeard = useRef("");
  // The listeners below are registered once, so they read the state through this.
  const stateRef = useRef(state);
  stateRef.current = state;
  // A question heard while a suggestion from the call is still being written waits
  // for it (`shouldQueueHeard`), then gets its own answer below it.
  const heardWaiting = useRef(false);

  // Follow the answer as it streams in, unless the user scrolled up to read.
  const threadRef = useRef<HTMLDivElement>(null);
  const following = useRef(true);
  useEffect(() => {
    const body = threadRef.current?.closest(".win-body");
    if (!(body instanceof HTMLElement)) return;
    const onScroll = () => {
      following.current = isNearBottom(body);
    };
    body.addEventListener("scroll", onScroll, { passive: true });
    return () => body.removeEventListener("scroll", onScroll);
  }, [state.phase === "idle"]);
  useEffect(() => {
    const body = threadRef.current?.closest(".win-body");
    if (body instanceof HTMLElement && following.current) body.scrollTop = body.scrollHeight;
  }, [state.answer, state.question, state.earlier.length, state.phase]);

  // The three answer events. Each `subscribe*` resolves to its unlisten function
  // asynchronously, so the cleanup copes with running before that happens.
  useEffect(() => {
    let cancelled = false;
    const stops: UnlistenFn[] = [];
    const keep = (stop: UnlistenFn) => (cancelled ? stop() : stops.push(stop));
    // A suggestion from the call: shown below the earlier ones, and followed.
    const suggestHeard = (): void => {
      following.current = true;
      dispatch({ type: "send", question: lastHeard.current, source: "heard" as AskSource });
      requestAnswer().catch((error: unknown) => dispatch({ type: "error", key: errorCopyKey(error) }));
    };
    // The suggestion being written ended: a question that waited for it goes next.
    const nextHeard = (): void => {
      if (!heardWaiting.current) return;
      heardWaiting.current = false;
      suggestHeard();
    };
    Promise.all([
      subscribeAnswerDelta((text) => dispatch({ type: "delta", text })).then(keep),
      subscribeAnswerDone(() => {
        dispatch({ type: "done" });
        nextHeard();
      }).then(keep),
      subscribeTranscriptDelta((turn) => {
        // The user's own words (microphone) are not what a suggestion answers.
        if (!turn.isUserTurn) lastHeard.current = turn.text;
      }).then(keep),
      // Rust took a screenshot and is answering what is on it.
      subscribeAnswerStarted(() => {
        following.current = true;
        dispatch({ type: "send", question: t("ask.screenQuestion"), source: "screen" });
      }).then(keep),
      // A suggestion from the call is starting: the questions it answers, as Rust
      // worked them out (everything since the last finished suggestion).
      subscribeAnswerPrompt((prompt) => dispatch({ type: "prompt", question: prompt })).then(keep),
      // Something on the call asks for a reply: suggest one without being asked. The
      // transcript is the context, so no question is sent. One still being written is
      // left to finish; the new question waits and is answered on its own after it.
      subscribeTurnDetected(() => {
        if (shouldQueueHeard(stateRef.current)) {
          heardWaiting.current = true;
          return;
        }
        suggestHeard();
      }).then(keep),
      // Shared with the realtime side; the reducer ignores it unless a question is in flight.
      subscribeError(() => {
        const wasBusy = isBusy(stateRef.current);
        dispatch({ type: "error", key: "errors.generic" });
        if (wasBusy) nextHeard();
      }).then(keep),
    ])
      // Every listener is registered: an answer started from now on will be shown.
      .then(() => announceAskReady())
      .catch((error: unknown) => {
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
    following.current = true;
    dispatch({ type: "send", question: question ?? "", source: "typed" });
    setInput("");
    try {
      await requestAnswer(question);
    } catch (error) {
      dispatch({ type: "error", key: errorCopyKey(error) });
    }
  }

  async function stop(): Promise<void> {
    heardWaiting.current = false;
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
        <div className="ask-thread" aria-live="polite" ref={threadRef}>
          {state.earlier.map((exchange, index) => (
            <div className="ask-exchange ask-earlier" key={index}>
              {exchange.question !== "" && (
                <section className="ask-block">
                  <h2 className="win-label">{t(sourceLabel(exchange.source))}</h2>
                  <p className="ask-question">{exchange.question}</p>
                </section>
              )}
              {exchange.answer !== "" && (
                <section className="ask-block">
                  <h2 className="win-label">{t("ask.suggestion")}</h2>
                  <div className="ask-answer">
                    <RichText text={exchange.answer} />
                  </div>
                </section>
              )}
            </div>
          ))}
          {state.question !== "" && (
            <section className="ask-block">
              <h2 className="win-label">{t(sourceLabel(state.source))}</h2>
              <p className="ask-question">{state.question}</p>
            </section>
          )}
          <section className="ask-block">
            <h2 className="win-label">{t("ask.suggestion")}</h2>
            {state.phase === "thinking" ? (
              <p className="win-muted">{t("ask.thinking")}</p>
            ) : (
              <div className="ask-answer">
                <RichText text={state.answer} />
              </div>
            )}
          </section>
        </div>
      )}
    </WindowFrame>
  );
}

/** The heading over a question: who asked it. */
function sourceLabel(source: AskSource): CatalogKey {
  return source === "heard" ? "ask.heard" : source === "screen" ? "ask.screen" : "ask.you";
}
