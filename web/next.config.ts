import type { NextConfig } from "next";
import { siteConfig } from "./src/site.config";

const nextConfig: NextConfig = {
  /* config options here */
  cacheComponents: true,
  partialPrefetching: true,
  // The website assistant reads the guide at runtime to answer from it
  // (src/lib/assistant-knowledge.ts): ship those files with its function.
  outputFileTracingIncludes: {
    "/api/assistant": ["./src/content/guide/**/*"],
  },
  turbopack: {
    // Fija la raíz al proyecto: evita que Next.js suba hasta ~/package-lock.json
    // (ajeno a este repo) al detectar el workspace root.
    root: __dirname,
  },
  async redirects() {
    return [
      // Descarga directa: los CTA del sitio apuntan a /dl/mac y este redirect
      // entrega el DMG de la última release. Un 302 externo a nivel de
      // plataforma: no abre la página de GitHub, no cuesta una función.
      // (Los redirects de next.config corren antes del proxy de locales.)
      {
        source: siteConfig.DOWNLOAD_URL,
        destination: siteConfig.DOWNLOAD_DMG_ARM64,
        permanent: false,
      },
      // Alias bajo /es: el enlace es el mismo en los dos idiomas, pero así una
      // URL escrita a mano (/es/dl/mac) tampoco muere en un 404.
      {
        source: `/es${siteConfig.DOWNLOAD_URL}`,
        destination: siteConfig.DOWNLOAD_DMG_ARM64,
        permanent: false,
      },
      {
        source: "/:path*",
        has: [{ type: "host", value: "www.getcloakview.com" }],
        destination: "https://getcloakview.com/:path*",
        permanent: true,
      },
      // El alias de producción de Vercel queda indexable si se sirve con 200:
      // Google lo trata como un segundo host con el mismo contenido. Un 301 al
      // dominio real lo consolida en una sola URL.
      {
        source: "/:path*",
        has: [{ type: "host", value: "cloakview.vercel.app" }],
        destination: "https://getcloakview.com/:path*",
        permanent: true,
      },
    ];
  },
  // Files in /public ship unhashed, so Next serves them with max-age=0. They only
  // change when a deploy replaces them; a year of immutable caching is safe for
  // the brand assets and removes a revalidation round-trip from every page.
  async headers() {
    return [
      {
        source: "/brand/:path*",
        headers: [{ key: "Cache-Control", value: "public, max-age=31536000, immutable" }],
      },
    ];
  },
};

export default nextConfig;
