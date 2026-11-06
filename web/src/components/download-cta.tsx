import type { Dictionary } from "@/lib/get-dictionary";
import { DownloadButton } from "@/components/download-button";

export function DownloadCta({
  dict,
}: {
  dict: Dictionary["home"]["download"];
}) {
  return (
    <section className="border-t border-border">
      <div className="mx-auto max-w-5xl px-6 py-20 text-center">
        <h2 className="text-3xl font-bold tracking-tight text-foreground">
          {dict.title}
        </h2>
        <p className="mx-auto mt-4 max-w-2xl text-lg leading-relaxed text-muted">
          {dict.body}
        </p>
        <div className="mt-8">
          <DownloadButton label={dict.button} soonLabel={dict.soon} />
        </div>
        <p className="mt-6 text-sm text-muted">{dict.note}</p>
      </div>
    </section>
  );
}
