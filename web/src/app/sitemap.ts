import type { MetadataRoute } from "next";
import { siteConfig } from "@/site.config";
import { guidePath, guideSlugs } from "@/lib/guide";
import changelog from "@/content/changelog.json";

const paths = ["/", "/pricing", "/download", "/faq", "/support", "/changelog", "/privacy", "/terms", "/cookies", guidePath("en"), ...guideSlugs.map((slug) => guidePath("en", slug))];

// The newest dated release: the changelog and the download page change when one ships.
const latestRelease = changelog
  .map((entry) => entry.date)
  .filter((date): date is string => typeof date === "string")
  .sort()
  .at(-1);

// Only dates we actually know. A build-time "now" on every URL tells search engines that
// everything changed on every deploy, so they learn to ignore lastmod; leaving it out is
// valid and lets them judge freshness themselves.
const lastModified: Partial<Record<string, string>> = {
  "/privacy": siteConfig.LEGAL_UPDATED,
  "/terms": siteConfig.LEGAL_UPDATED,
  "/cookies": siteConfig.LEGAL_UPDATED,
  "/changelog": latestRelease,
  "/download": latestRelease,
};

export default function sitemap(): MetadataRoute.Sitemap {
  return paths.flatMap((path) => [
    ...(["en", "es"] as const).map((locale) => ({
      url: `${siteConfig.url}${locale === "en" ? path : `/es${path === "/" ? "" : path}`}`,
      ...(lastModified[path] ? { lastModified: lastModified[path] } : {}),
      alternates: {
        languages: {
          en: `${siteConfig.url}${path}`,
          es: `${siteConfig.url}/es${path === "/" ? "" : path}`,
          "x-default": `${siteConfig.url}${path}`,
        },
      },
    })),
  ]);
}
