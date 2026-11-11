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
};

export default nextConfig;
