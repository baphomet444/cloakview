import type { Dictionary } from "@/lib/get-dictionary";

export function RoadmapNote({
  dict,
}: {
  dict: Dictionary["pricing"]["roadmap"];
}) {
  return (
    <section className="border-t border-border">
      <div className="mx-auto max-w-5xl px-6 py-20">
        <h2 className="font-display text-3xl font-bold tracking-tight text-foreground">
          {dict.title}
        </h2>
        <p className="mt-6 max-w-3xl text-lg leading-relaxed text-muted">
          {dict.body}
        </p>
      </div>
    </section>
  );
}
