"use client";

import { useEffect, useState } from "react";

export function GuideToc({ sections, label }: { sections: { id: string; title: string }[]; label: string }) {
  const [active, setActive] = useState(sections[0]?.id);
  useEffect(() => {
    const update = () => {
      const current = sections.filter(({ id }) => {
        const element = document.getElementById(id);
        return element && element.getBoundingClientRect().top <= 110;
      }).at(-1);
      setActive(current?.id ?? sections[0]?.id);
    };
    update();
    window.addEventListener("scroll", update, { passive: true });
    window.addEventListener("resize", update);
    return () => { window.removeEventListener("scroll", update); window.removeEventListener("resize", update); };
  }, [sections]);
  return (
    <nav aria-label={label} className="rounded-xl border border-border bg-surface p-5 lg:sticky lg:top-8 lg:self-start">
      <p className="mb-4 font-semibold text-foreground">{label}</p>
      <ul className="space-y-2 text-sm">
        {sections.map(({ id, title }) => (
          <li key={id}><a href={`#${id}`} onClick={() => setActive(id)} aria-current={active === id ? "location" : undefined} className={`block rounded px-2 py-1.5 focus-visible:outline-2 focus-visible:outline-accent ${active === id ? "bg-accent text-accent-foreground" : "text-muted hover:text-foreground"}`}>{title}</a></li>
        ))}
      </ul>
    </nav>
  );
}
