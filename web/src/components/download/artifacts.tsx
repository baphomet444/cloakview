"use client";

import { useEffect, useState } from "react";
import { DownloadButton } from "@/components/download-button";
import type { Dictionary } from "@/lib/get-dictionary";
import { isGithubPending, siteConfig } from "@/site.config";

type DownloadCopy = Dictionary["download"];
type Artifact = DownloadCopy["artifacts"][number];

function ShaBlock({ artifact, hash, copy }: { artifact: Artifact; hash: string | null; copy: DownloadCopy }) {
  const [feedback, setFeedback] = useState("");
  const available = Boolean(hash);

  async function copyHash() {
    if (!hash || !available) return;
    try {
      await navigator.clipboard.writeText(hash);
      setFeedback(copy.copied);
    } catch {
      setFeedback(copy.copyFailed);
    }
  }

  return (
    <div className="mt-6 border-t border-border pt-5">
      <p className="text-sm font-medium text-foreground">{copy.shaLabel}</p>
      <div className="mt-2 flex flex-wrap items-center gap-3">
        <code className={`break-all font-mono text-xs ${available ? "text-foreground" : "text-muted"}`}>
          {available ? hash : copy.shaPending}
        </code>
        <button type="button" onClick={copyHash} disabled={!available}
          aria-label={`${copy.copyLabel} ${artifact.name}`}
          className="rounded border border-border px-3 py-1 text-sm text-foreground hover:border-accent disabled:cursor-not-allowed disabled:opacity-50">
          {copy.copyLabel}
        </button>
      </div>
      <span role="status" aria-live="polite" className="text-sm text-muted">{feedback}</span>
      {available && !isGithubPending() && (
        <a href={siteConfig.GITHUB_RELEASES} rel="noopener noreferrer" className="mt-2 block w-fit text-sm text-accent underline underline-offset-4">
          {copy.verifyGithub}
        </a>
      )}
      {(isGithubPending() || !available) && <p className="mt-2 text-sm text-muted">{copy.verifyPending}</p>}
    </div>
  );
}

export function Artifacts({ dict }: { dict: DownloadCopy }) {
  const [isArm, setIsArm] = useState(false);
  useEffect(() => {
    // User-Agent Client Hints reports architecture only when supported; no guess
    // from a generic Mac user agent (which can represent either architecture).
    const agent = navigator as Navigator & {
      userAgentData?: { platform: string; getHighEntropyValues: (hints: string[]) => Promise<{ architecture?: string }> };
    };
    if (agent.userAgentData?.platform !== "macOS") return;
    let active = true;
    agent.userAgentData.getHighEntropyValues(["architecture"])
      .then(({ architecture }) => { if (active && (architecture === "arm" || architecture === "arm64")) setIsArm(true); })
      .catch(() => {});
    return () => { active = false; };
  }, []);

  const hashes = [siteConfig.SHA256_ARM64, siteConfig.SHA256_INTEL, siteConfig.SHA256_WINDOWS];
  return (
    <section aria-label={dict.optionsLabel} className="border-t border-border">
      <div className="mx-auto grid max-w-5xl gap-5 px-6 py-16 md:grid-cols-3">
        {dict.artifacts.map((artifact, index) => (
          <article key={artifact.name} className={`rounded-xl border p-6 ${index === 0 && isArm ? "border-accent bg-surface" : "border-border bg-surface"}`}>
            {index === 0 && isArm && <p className="mb-3 text-sm font-medium text-accent">{dict.recommended}</p>}
            <h2 className="font-display text-xl font-semibold text-foreground">{artifact.name}</h2>
            <p className="mt-2 min-h-12 text-sm leading-relaxed text-muted">{artifact.description}</p>
            {index < 2 && <p className="mt-4 text-sm text-foreground">{dict.notarized}</p>}
            <div className="mt-6">
              {index === 0 ? (
                <DownloadButton label={artifact.button} soonLabel={dict.soon} />
              ) : (
                <span aria-disabled="true" className="inline-flex cursor-not-allowed rounded-lg border border-border px-5 py-2.5 text-sm text-muted">
                  {artifact.unavailable}
                </span>
              )}
            </div>
            <ShaBlock artifact={artifact} hash={hashes[index]} copy={dict} />
          </article>
        ))}
      </div>
    </section>
  );
}
