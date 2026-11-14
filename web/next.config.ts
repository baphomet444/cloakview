import type { NextConfig } from "next";

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
      {
        source: "/:path*",
        has: [{ type: "host", value: "www.getcloakview.com" }],
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
