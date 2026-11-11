"use client";
import { useState } from "react";
import Link from "next/link";
import type { Dictionary } from "@/lib/get-dictionary";
import { guidePath, isGuideSlug } from "@/lib/guide-path";
import type { Locale } from "@/site.config";

export function FaqTopics({ faq, locale }: { faq: Dictionary["faq"]; locale: Locale }) {
  const [open, setOpen] = useState<string | null>(null);
  return <div className="space-y-12">
    {faq.topics.map((topic, topicIndex) => <section key={topic.title} aria-labelledby={`topic-${topicIndex}`}>
      <h2 id={`topic-${topicIndex}`} className="font-display mb-5 text-2xl font-semibold">{topic.title}</h2>
      <div className="space-y-3">{topic.items.map((item, itemIndex) => {
        const id = `faq-${topicIndex}-${itemIndex}`;
        const expanded = open === id;
        return <div key={id} className="rounded-xl border border-border bg-surface">
          <h3><button id={`${id}-button`} type="button" aria-expanded={expanded} aria-controls={`${id}-panel`} onClick={() => setOpen(expanded ? null : id)} className="flex w-full items-center justify-between gap-4 rounded-xl px-5 py-4 text-left font-medium text-foreground hover:text-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent">
            {item.q}<span aria-hidden="true" className="text-accent">{expanded ? "−" : "+"}</span>
          </button></h3>
          <div id={`${id}-panel`} role="region" aria-labelledby={`${id}-button`} hidden={!expanded} className="px-5 pb-5 text-muted">
            <p>{item.a}</p>
            {item.guide && isGuideSlug(item.guide) && <Link className="mt-3 inline-block text-accent underline underline-offset-4 focus-visible:outline-2 focus-visible:outline-accent" href={guidePath(locale, item.guide)}>{faq.guideLink}</Link>}
          </div>
        </div>;
      })}</div>
    </section>)}
  </div>;
}
