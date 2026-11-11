import "server-only";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { OPENAI_API_KEYS, OPENAI_STATUS, siteConfig, type Locale } from "@/site.config";
import { getDictionary } from "@/lib/get-dictionary";
import { guidePath, guideSlugs } from "@/lib/guide-path";

/**
 * What the assistant may answer from: the published guide, FAQ and pricing copy in the
 * visitor's language, as plain text. Built from the same files the site renders, so the
 * assistant never drifts from what the pages say. The guide files are read at runtime;
 * `next.config.ts` (`outputFileTracingIncludes`) ships them with the API route.
 */
const cache = new Map<Locale, Promise<string>>();

export function assistantKnowledge(locale: Locale): Promise<string> {
  let built = cache.get(locale);
  if (built === undefined) {
    built = build(locale);
    cache.set(locale, built);
  }
  return built;
}

async function build(locale: Locale): Promise<string> {
  const dict = await getDictionary(locale);
  const parts: string[] = [];

  for (const slug of guideSlugs) {
    const source = await readFile(path.join(process.cwd(), "src/content/guide", locale, `${slug}.mdx`), "utf8");
    const text = source
      .replaceAll("OPENAI_API_KEYS", OPENAI_API_KEYS)
      .replaceAll("OPENAI_STATUS", OPENAI_STATUS)
      .replace(/^## (.+) \{#[a-z0-9-]+\}$/gm, "### $1");
    parts.push(`## Guide: ${slug} (${siteConfig.url}${guidePath(locale, slug)})\n\n${text.trim()}`);
  }

  const faq = dict.faq.topics
    .flatMap((topic) => topic.items.map((item) => `Q: ${item.q}\nA: ${item.a}`))
    .join("\n\n");
  parts.push(`## FAQ\n\n${faq}`);
  parts.push(`## Pricing\n\n${dict.pricing.description}`);
  parts.push(`## Download\n\n${dict.download.description}`);

  return parts.join("\n\n---\n\n");
}
