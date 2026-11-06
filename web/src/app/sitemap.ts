import type { MetadataRoute } from "next";
import { siteConfig } from "@/site.config";

// Fase 1: solo la home existe. Cada fase posterior que añada una ruta debe
// añadir su entrada aquí (en/es) para mantener el sitemap sin huecos.
const paths = ["/", "/pricing"];

export default function sitemap(): MetadataRoute.Sitemap {
  const now = new Date();
  return paths.flatMap((path) => [
    {
      url: `${siteConfig.url}${path}`,
      lastModified: now,
      alternates: {
        languages: {
          en: `${siteConfig.url}${path}`,
          es: `${siteConfig.url}/es${path === "/" ? "" : path}`,
        },
      },
    },
  ]);
}
