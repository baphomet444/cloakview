import "server-only";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { evaluate } from "@mdx-js/mdx";
import * as runtime from "react/jsx-runtime";
import { createElement, type ComponentType, type ReactNode } from "react";
import { codeToHtml } from "shiki";
import type { Locale } from "@/site.config";
import { OPENAI_API_KEYS, OPENAI_STATUS } from "@/site.config";
import { CodeBlock } from "@/components/guide/code-block";

export const guideSlugs = ["installation", "permissions", "api-key", "shortcuts", "troubleshooting"] as const;
export type GuideSlug = (typeof guideSlugs)[number];
export function isGuideSlug(value: string): value is GuideSlug {
  return (guideSlugs as readonly string[]).includes(value);
}

export const guidePath = (locale: Locale, slug?: GuideSlug) =>
  `${locale === "en" ? "" : "/es"}/guide${slug ? `/${slug}` : ""}`;

export async function loadArticle(locale: Locale, slug: GuideSlug, copyLabel: string, copiedLabel: string, errorLabel: string) {
  "use cache";
  // Slug and locale are validated against fixed unions before reading files.
  const source = await readFile(path.join(process.cwd(), "src/content/guide", locale, `${slug}.mdx`), "utf8");
  const sections = [...source.matchAll(/^## (.+) \{#([a-z0-9-]+)\}$/gm)].map((match) => ({ title: match[1], id: match[2] }));
  if (sections.length === 0 || new Set(sections.map((s) => s.id)).size !== sections.length) {
    throw new Error(`Guide sections missing or duplicated: ${locale}/${slug}`);
  }
  // Only committed local MDX is evaluated, never user-supplied input.
  const normalizedSource = source
    .replaceAll("OPENAI_API_KEYS", OPENAI_API_KEYS)
    .replaceAll("OPENAI_STATUS", OPENAI_STATUS);
  const { default: Article } = await evaluate(normalizedSource.replace(/^## (.+) \{#([a-z0-9-]+)\}$/gm, '<h2 id="$2">$1</h2>'), { ...runtime, development: false });
  const components = {
    h2: ({ children, id }: { children?: ReactNode; id?: string }) =>
      createElement("h2", { id, className: "scroll-mt-24 mt-12 mb-4 text-2xl font-semibold text-foreground" }, children),
    a: ({ href, children }: { href?: string; children?: ReactNode }) =>
      createElement("a", { href, rel: href?.startsWith("http") ? "noopener noreferrer" : undefined, className: "text-accent underline underline-offset-4 hover:text-foreground" }, children),
    pre: async ({ children }: { children?: ReactNode }) => {
      const code = children as { props?: { children?: string; className?: string } };
      const raw = String(code?.props?.children ?? "").replace(/\n$/, "");
      const lang = code?.props?.className?.replace(/^language-/, "") || "text";
      const html = await codeToHtml(raw, { lang, theme: "github-dark" });
      return createElement(CodeBlock, { html, raw, language: lang, copyLabel, copiedLabel, errorLabel });
    },
  };
  return { sections, content: createElement(Article as ComponentType<{ components: typeof components }>, { components }) };
}
