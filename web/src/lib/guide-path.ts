import type { Locale } from "@/site.config";

export const guideSlugs = ["installation", "permissions", "api-key", "shortcuts", "troubleshooting"] as const;
export type GuideSlug = (typeof guideSlugs)[number];
export function isGuideSlug(value: string): value is GuideSlug {
  return (guideSlugs as readonly string[]).includes(value);
}
export const guidePath = (locale: Locale, slug?: GuideSlug) =>
  `${locale === "en" ? "" : "/es"}/guide${slug ? `/${slug}` : ""}`;
