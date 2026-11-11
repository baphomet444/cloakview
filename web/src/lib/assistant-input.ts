// Entrada del asistente de la web: validación, borrado de claves y límite de peticiones.
// Sin dependencias ni alias de ruta, para poder probarlo con node:test (tests/assistant.test.ts).

export type AssistantLocale = "en" | "es";
export type AssistantMessage = { role: "user" | "assistant"; content: string };

export const ASSISTANT_LIMITS = {
  /** Longest single message a visitor can send. */
  maxMessageChars: 800,
  /** Turns of history sent to the model; older turns are the client's to drop. */
  maxMessages: 10,
} as const;

/**
 * OpenAI secret keys (`sk-…`, `sk-proj-…`, `sk-svcacct-…`). A visitor may paste one while
 * asking how to connect it; it must never reach the model, the logs or the provider.
 */
const OPENAI_KEY = /\bsk-[A-Za-z0-9_-]{20,}/g;
const REDACTED = "[redacted-key]";

export function redactSecrets(text: string): string {
  return text.replace(OPENAI_KEY, REDACTED);
}

export type ParsedAssistantRequest =
  | { ok: true; locale: AssistantLocale; messages: AssistantMessage[]; redacted: boolean }
  | { ok: false; reason: string };

/** Checks an untrusted request body. The last message must be the visitor's question. */
export function parseAssistantRequest(body: unknown): ParsedAssistantRequest {
  if (typeof body !== "object" || body === null) return { ok: false, reason: "body" };
  const { locale, messages } = body as { locale?: unknown; messages?: unknown };
  if (!Array.isArray(messages) || messages.length === 0) return { ok: false, reason: "messages" };
  if (messages.length > ASSISTANT_LIMITS.maxMessages) return { ok: false, reason: "too-many" };

  const clean: AssistantMessage[] = [];
  let redacted = false;
  for (const raw of messages) {
    if (typeof raw !== "object" || raw === null) return { ok: false, reason: "message" };
    const { role, content } = raw as { role?: unknown; content?: unknown };
    if (role !== "user" && role !== "assistant") return { ok: false, reason: "role" };
    if (typeof content !== "string" || content.trim() === "") return { ok: false, reason: "content" };
    if (content.length > ASSISTANT_LIMITS.maxMessageChars) return { ok: false, reason: "too-long" };
    const safe = redactSecrets(content);
    if (safe !== content) redacted = true;
    clean.push({ role, content: safe });
  }
  if (clean[clean.length - 1].role !== "user") return { ok: false, reason: "last" };

  return { ok: true, locale: locale === "es" ? "es" : "en", messages: clean, redacted };
}

/**
 * A fixed-window limiter keyed by client (IP). In memory, so it is per server instance:
 * a first line against a single visitor hammering the endpoint, not a global quota. The
 * hard ceiling on spend is the AI Gateway budget.
 */
export function createRateLimiter(options: { max: number; windowMs: number; now?: () => number }) {
  const now = options.now ?? Date.now;
  const windows = new Map<string, { start: number; count: number }>();
  return function allow(key: string): boolean {
    const t = now();
    const current = windows.get(key);
    if (current === undefined || t - current.start > options.windowMs) {
      windows.set(key, { start: t, count: 1 });
      if (windows.size > 10_000) windows.clear(); // never grow without bound
      return true;
    }
    current.count += 1;
    return current.count <= options.max;
  };
}
