"use client";

import { useEffect, useRef, useState, type FormEvent, type ReactNode } from "react";
import type { Dictionary } from "@/lib/get-dictionary";
import type { Locale } from "@/site.config";
import { ASSISTANT_LIMITS } from "@/lib/assistant-input";

type AssistantDict = Dictionary["assistant"];
type Topic = { q: string; a: string; steps?: readonly string[]; note?: string; linkLabel: string; href: string };

type Message =
  | { role: "user"; content: string }
  | { role: "assistant"; content: string; topic?: Topic; status?: "streaming" | "error" };

/** Turns full URLs in the model's plain-text answer into links; everything else stays text. */
function linkify(text: string): ReactNode[] {
  return text.split(/(https?:\/\/[^\s)]+)/g).map((part, i) =>
    /^https?:\/\//.test(part) ? (
      <a key={i} href={part.replace(/[.,;:]$/, "")} target="_blank" rel="noopener noreferrer" className="text-accent underline underline-offset-2">
        {part}
      </a>
    ) : (
      part
    ),
  );
}

/**
 * The site's help assistant. The popular questions are answered from the dictionary,
 * instantly and without any request. Only a question the visitor types goes to
 * `/api/assistant`, which answers with AI from the guide and FAQ.
 */
export function AssistantWidget({ dict, locale }: { dict: AssistantDict; locale: Locale }) {
  const [open, setOpen] = useState(false);
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [busy, setBusy] = useState(false);
  const panelRef = useRef<HTMLDivElement>(null);
  const endRef = useRef<HTMLDivElement>(null);
  const abortRef = useRef<AbortController | null>(null);
  const topics: readonly Topic[] = dict.topics;

  useEffect(() => {
    endRef.current?.scrollIntoView({ block: "end" });
  }, [messages]);

  useEffect(() => {
    if (!open) return;
    panelRef.current?.focus();
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(false);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open]);

  useEffect(() => () => abortRef.current?.abort(), []);

  function askTopic(topic: Topic) {
    setMessages((m) => [...m, { role: "user", content: topic.q }, { role: "assistant", content: topic.a, topic }]);
  }

  async function askFree(event: FormEvent) {
    event.preventDefault();
    const question = input.trim();
    if (question === "" || busy) return;
    setInput("");
    setBusy(true);

    // History as plain text: a guided answer is sent as its text and steps.
    const history = [...messages, { role: "user", content: question } as const]
      .map((m) => ({
        role: m.role,
        content: m.role === "assistant" && m.topic ? [m.topic.a, ...(m.topic.steps ?? []), m.topic.note ?? ""].join("\n") : m.content,
      }))
      .filter((m) => m.content.trim() !== "" && !(m.role === "assistant" && m.content === dict.error))
      .slice(-ASSISTANT_LIMITS.maxMessages);

    setMessages((m) => [...m, { role: "user", content: question }, { role: "assistant", content: "", status: "streaming" }]);
    const setLast = (update: (last: Message) => Message) =>
      setMessages((m) => [...m.slice(0, -1), update(m[m.length - 1])]);

    const controller = new AbortController();
    abortRef.current = controller;
    try {
      const response = await fetch("/api/assistant", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ locale, messages: history }),
        signal: controller.signal,
      });
      if (!response.ok || response.body === null) {
        const text = response.status === 429 ? dict.rateLimited : dict.error;
        setLast(() => ({ role: "assistant", content: text, status: "error" }));
        return;
      }
      const reader = response.body.getReader();
      const decoder = new TextDecoder();
      let answer = "";
      for (;;) {
        const { done, value } = await reader.read();
        if (done) break;
        answer += decoder.decode(value, { stream: true });
        const current = answer;
        setLast(() => ({ role: "assistant", content: current, status: "streaming" }));
      }
      // An empty stream means the model call failed after the response started.
      setLast(() =>
        answer.trim() === "" ? { role: "assistant", content: dict.error, status: "error" } : { role: "assistant", content: answer },
      );
    } catch {
      if (!controller.signal.aborted) setLast(() => ({ role: "assistant", content: dict.error, status: "error" }));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="fixed right-4 bottom-4 z-50 flex flex-col items-end gap-3 sm:right-6 sm:bottom-6">
      {open && (
        <div
          ref={panelRef}
          role="dialog"
          aria-modal="false"
          aria-label={dict.title}
          tabIndex={-1}
          className="flex h-[min(34rem,calc(100dvh-6rem))] w-[min(24rem,calc(100vw-2rem))] flex-col overflow-hidden rounded-2xl border border-border bg-surface text-foreground shadow-2xl shadow-black/50 outline-none"
        >
          <div className="flex items-center justify-between border-b border-border px-4 py-3">
            <p className="flex items-center gap-2 font-display text-sm">
              <span className="size-2 rounded-full bg-accent" aria-hidden="true" />
              {dict.title}
            </p>
            <button type="button" onClick={() => setOpen(false)} aria-label={dict.close} className="rounded-md px-2 py-1 text-muted hover:text-foreground">
              ✕
            </button>
          </div>

          <div className="flex-1 space-y-3 overflow-y-auto px-4 py-4 text-sm leading-relaxed" aria-live="polite">
            <p className="text-muted">{dict.intro}</p>
            <div>
              <p className="mb-2 text-xs tracking-wide text-muted uppercase">{dict.quickTitle}</p>
              <div className="flex flex-wrap gap-2">
                {topics.map((topic) => (
                  <button
                    key={topic.q}
                    type="button"
                    onClick={() => askTopic(topic)}
                    className="rounded-full border border-border px-3 py-1.5 text-left text-xs text-foreground transition-colors hover:border-accent hover:text-accent"
                  >
                    {topic.q}
                  </button>
                ))}
              </div>
            </div>

            {messages.map((message, i) =>
              message.role === "user" ? (
                <p key={i} className="ml-8 rounded-2xl rounded-br-sm bg-accent px-3 py-2 text-accent-foreground">
                  {message.content}
                </p>
              ) : (
                <div key={i} className={`mr-4 rounded-2xl rounded-bl-sm border border-border px-3 py-2 ${message.status === "error" ? "text-muted" : ""}`}>
                  {message.status === "streaming" && message.content === "" ? (
                    <p className="text-muted">{dict.thinking}</p>
                  ) : (
                    <p className="whitespace-pre-wrap">{linkify(message.content)}</p>
                  )}
                  {message.topic?.steps && (
                    <ol className="mt-2 list-decimal space-y-1.5 pl-5 marker:text-accent">
                      {message.topic.steps.map((step) => (
                        <li key={step}>{step}</li>
                      ))}
                    </ol>
                  )}
                  {message.topic?.note && <p className="mt-2 text-muted">{message.topic.note}</p>}
                  {message.topic && (
                    <a href={message.topic.href} className="mt-2 inline-block text-accent underline underline-offset-2">
                      {message.topic.linkLabel} →
                    </a>
                  )}
                  {message.status === "error" && (
                    <a href={dict.supportHref} className="mt-2 block text-accent underline underline-offset-2">
                      {dict.supportLabel}
                    </a>
                  )}
                </div>
              ),
            )}
            <div ref={endRef} />
          </div>

          <form onSubmit={askFree} className="border-t border-border px-3 pt-3 pb-2">
            <div className="flex gap-2">
              <label htmlFor="assistant-input" className="sr-only">
                {dict.inputLabel}
              </label>
              <input
                id="assistant-input"
                value={input}
                onChange={(event) => setInput(event.target.value)}
                maxLength={ASSISTANT_LIMITS.maxMessageChars}
                placeholder={dict.placeholder}
                autoComplete="off"
                className="min-w-0 flex-1 rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted focus:border-accent focus:outline-none"
              />
              <button
                type="submit"
                disabled={busy || input.trim() === ""}
                className="rounded-lg bg-accent px-3 py-2 text-sm font-semibold text-accent-foreground disabled:opacity-40"
              >
                {dict.send}
              </button>
            </div>
            <p className="mt-2 text-[11px] leading-snug text-muted">{dict.aiNotice}</p>
          </form>
        </div>
      )}

      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        aria-expanded={open}
        aria-label={open ? dict.close : dict.open}
        className="flex items-center gap-2 rounded-full bg-accent px-4 py-3 text-sm font-semibold text-accent-foreground shadow-lg shadow-accent/30 transition-transform hover:scale-105"
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path d="M4 5h16v11H8l-4 4V5z" stroke="currentColor" strokeWidth="2" strokeLinejoin="round" />
        </svg>
        <span className="hidden sm:inline">{open ? dict.close : dict.open}</span>
      </button>
    </div>
  );
}
