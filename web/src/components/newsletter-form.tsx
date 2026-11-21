"use client";

import { useState, type FormEvent } from "react";
import { track } from "@vercel/analytics";
import type { Dictionary } from "@/lib/get-dictionary";
import type { Locale } from "@/site.config";

type Status = "idle" | "sending" | "done" | "invalid" | "error" | "unavailable";

/**
 * Alta en la lista de avisos de CloakView Pro. Es opcional y vive fuera del flujo de
 * descarga: el sitio promete no pedir email para bajar el DMG y esta sección no lo
 * contradice, solo ofrece un aviso cuando salga Pro.
 *
 * Todo el POST va a `/api/subscribe`, que reenvía al proveedor real y responde 503 si
 * la lista todavía no está conectada; el formulario muestra ese caso como "todavía no".
 */
export function NewsletterForm({
  dict,
  locale,
  source,
}: {
  dict: Dictionary["newsletter"];
  locale: Locale;
  source: string;
}) {
  const [email, setEmail] = useState("");
  const [status, setStatus] = useState<Status>("idle");

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (status === "sending" || status === "done") return;
    setStatus("sending");
    try {
      const response = await fetch("/api/subscribe", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email, locale }),
      });
      if (response.ok) {
        setStatus("done");
        setEmail("");
        track("newsletter_signup", { from: source });
        return;
      }
      if (response.status === 400) return setStatus("invalid");
      if (response.status === 503) return setStatus("unavailable");
      setStatus("error");
    } catch {
      setStatus("error");
    }
  }

  const message =
    status === "done"
      ? dict.success
      : status === "invalid"
        ? dict.invalid
        : status === "error"
          ? dict.error
          : status === "unavailable"
            ? dict.unavailable
            : "";

  return (
    <section className="border-t border-border">
      <div className="mx-auto max-w-5xl px-6 py-20 text-center">
        <h2 className="font-display text-3xl font-bold tracking-tight text-foreground">
          {dict.title}
        </h2>
        <p className="mx-auto mt-4 max-w-2xl text-lg leading-relaxed text-muted">{dict.body}</p>
        <form
          onSubmit={onSubmit}
          className="mx-auto mt-8 flex w-full max-w-md flex-col gap-3 sm:flex-row"
        >
          <label htmlFor={`newsletter-email-${source}`} className="sr-only">
            {dict.placeholder}
          </label>
          <input
            id={`newsletter-email-${source}`}
            type="email"
            name="email"
            required
            autoComplete="email"
            inputMode="email"
            value={email}
            onChange={(event) => setEmail(event.target.value)}
            placeholder={dict.placeholder}
            disabled={status === "sending" || status === "done"}
            className="min-w-0 flex-1 rounded-lg border border-border bg-surface px-4 py-2.5 text-foreground placeholder:text-muted/70 focus:border-accent focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/40 disabled:opacity-60"
          />
          <button
            type="submit"
            disabled={status === "sending" || status === "done"}
            className="btn btn-primary shrink-0 disabled:cursor-not-allowed disabled:opacity-60"
          >
            {status === "sending" ? dict.sending : status === "done" ? dict.done : dict.button}
          </button>
        </form>
        <p aria-live="polite" role="status" className="mt-4 min-h-5 text-sm text-muted">
          {message}
        </p>
        <p className="mx-auto mt-2 max-w-xl text-xs leading-relaxed text-muted/80">{dict.note}</p>
      </div>
    </section>
  );
}
