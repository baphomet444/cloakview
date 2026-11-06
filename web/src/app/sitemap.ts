import type { MetadataRoute } from "next";
import { siteConfig } from "@/site.config";
import { guidePath, guideSlugs } from "@/lib/guide";

const paths = ["/", "/pricing", "/download", guidePath("en"), ...guideSlugs.map((slug) => guidePath("en", slug))];

export default function sitemap(): MetadataRoute.Sitemap {
  const now = new Date();
  return paths.flatMap((path) => [
    ...(["en", "es"] as const).map((locale) => ({
      url: `${siteConfig.url}${locale === "en" ? path : `/es${path === "/" ? "" : path}`}`,
      lastModified: now,
      alternates: {
        languages: {
          en: `${siteConfig.url}${path}`,
          es: `${siteConfig.url}/es${path === "/" ? "" : path}`,
        },
      },
    })),
  ]);
}
