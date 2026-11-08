import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  /* config options here */
  cacheComponents: true,
  partialPrefetching: true,
  turbopack: {
    // Fija la raíz al proyecto: evita que Next.js suba hasta ~/package-lock.json
    // (ajeno a este repo) al detectar el workspace root.
    root: __dirname,
  },
};

export default nextConfig;
