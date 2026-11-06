import type { Dictionary } from "@/lib/get-dictionary";

export function CostEstimator({
  dict,
}: {
  dict: Dictionary["pricing"]["estimator"];
}) {
  return (
    <section className="border-t border-border">
      <div className="mx-auto max-w-5xl px-6 py-20">
        <h2 className="text-3xl font-bold tracking-tight text-foreground">
          {dict.title}
        </h2>
        <p className="mt-3 text-lg text-muted">{dict.subtitle}</p>
        <ul className="mt-12 grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {dict.profiles.map((profile) => (
            <li key={profile.name} className="rounded-xl border border-border p-6">
              <h3 className="text-lg font-semibold text-foreground">
                {profile.name}
              </h3>
              <p className="mt-1 text-sm text-muted">{profile.hours}</p>
              <p className="mt-4 text-2xl font-bold tracking-tight text-foreground">
                {profile.range}
              </p>
              <p className="mt-3 text-sm leading-relaxed text-muted">
                {profile.desc}
              </p>
            </li>
          ))}
        </ul>
        <p className="mt-8 text-sm leading-relaxed text-muted">
          {dict.disclaimer}
        </p>
        <p className="mt-4 text-sm leading-relaxed text-muted">{dict.footnote}</p>
      </div>
    </section>
  );
}
