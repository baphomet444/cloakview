import type { Dictionary } from "@/lib/get-dictionary";
import { DownloadButton } from "@/components/download-button";

export function DownloadCta({
  dict,
}: {
  dict: Dictionary["home"]["download"];
}) {
  return (
    <section className="border-t border-border">
      <div className="mx-auto max-w-7xl px-6 py-24 text-center lg:py-32">
        <h2 className="font-display text-5xl leading-[0.95] font-semibold tracking-[-0.025em] text-foreground sm:text-6xl">
          {dict.title}
        </h2>
        <p className="mx-auto mt-6 max-w-2xl text-lg leading-relaxed text-muted">
          {dict.body}
        </p>
        <div className="mt-9">
          <DownloadButton label={dict.button} soonLabel={dict.soon} />
        </div>
        <p className="mt-6 text-sm text-muted">{dict.note}</p>
      </div>
    </section>
  );
}
