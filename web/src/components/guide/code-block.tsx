"use client";

import { useState } from "react";

export function CodeBlock({ html, raw, language, copyLabel, copiedLabel, errorLabel }: {
  html: string; raw: string; language: string; copyLabel: string; copiedLabel: string; errorLabel: string;
}) {
  const [status, setStatus] = useState("");
  async function copy() {
    try {
      await navigator.clipboard.writeText(raw);
      setStatus(copiedLabel);
    } catch {
      setStatus(errorLabel);
    }
  }
  return (
    <div className="my-5 overflow-hidden rounded-xl border border-border bg-surface">
      <div className="flex items-center justify-between border-b border-border px-4 py-2 text-sm text-muted">
        <span>{language}</span>
        <button type="button" onClick={copy} className="rounded border border-border px-3 py-1 text-foreground hover:border-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent" aria-label={copyLabel}>{copyLabel}</button>
      </div>
      <div className="overflow-x-auto p-4 text-sm [&_pre]:w-max [&_pre]:min-w-full"><div dangerouslySetInnerHTML={{ __html: html }} /></div>
      <span role="status" aria-live="polite" className="sr-only">{status}</span>
    </div>
  );
}
