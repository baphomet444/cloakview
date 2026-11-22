import { createRateLimiter } from "@/lib/assistant-input";
import { mint, verify } from "@/lib/dl-token";

/**
 * The only public download URL.
 *
 * Release assets on a private repository answer 404 to anonymous callers, so the download
 * button cannot point at GitHub once the repo is private. This fronts the artifact instead,
 * in two hops: /api/dl mints a short-lived signed token and redirects to itself, so the URL
 * the browser ends up on is unguessable and expires, and the binary never has a stable
 * public path.
 *
 * Backends, by environment:
 *   DL_PRESIGN_URL  set -> redirect to it (R2/S3 presigned; storage stays private)
 *   DL_GITHUB_TOKEN set -> stream the asset through this function (private repo)
 *   neither             -> redirect to browser_download_url (repo still public)
 *
 * No route segment config: next.config sets cacheComponents, which rejects `runtime` and
 * `dynamic`. Route handlers are dynamic and Node-runtime by default, which is what this needs
 * (node:crypto in dl-token).
 */

const REPO = process.env.DL_REPO ?? "baphomet444/cloakview";
const ASSET = process.env.DL_ASSET ?? "CloakView-Pro_aarch64.dmg";
const GITHUB_TOKEN = process.env.DL_GITHUB_TOKEN;
const PRESIGN = process.env.DL_PRESIGN_URL;

const allow = createRateLimiter({ max: 30, windowMs: 60_000 });

type Asset = { url: string; browser_download_url: string };
let cached: { at: number; asset: Asset | null } = { at: 0, asset: null };

async function latestAsset(): Promise<Asset | null> {
  if (Date.now() - cached.at < 5 * 60 * 1000) return cached.asset;
  const res = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`, {
    headers: {
      Accept: "application/vnd.github+json",
      ...(GITHUB_TOKEN ? { Authorization: `Bearer ${GITHUB_TOKEN}` } : {}),
    },
    cache: "no-store",
  });
  if (!res.ok) {
    cached = { at: Date.now(), asset: null };
    return null;
  }
  const body = (await res.json()) as { assets?: (Asset & { name: string })[] };
  const asset = body.assets?.find((candidate) => candidate.name === ASSET) ?? null;
  cached = { at: Date.now(), asset };
  return asset;
}

export async function GET(request: Request): Promise<Response> {
  const ip = request.headers.get("x-forwarded-for")?.split(",")[0]?.trim() || "unknown";
  if (!allow(ip)) return Response.json({ error: "rate-limited" }, { status: 429 });

  const url = new URL(request.url);
  const token = url.searchParams.get("t");

  if (!token) {
    const next = new URL(url);
    next.searchParams.set("t", mint());
    return new Response(null, { status: 302, headers: { Location: next.toString(), "Cache-Control": "no-store" } });
  }
  if (!verify(token)) return Response.json({ error: "expired" }, { status: 403 });

  if (PRESIGN) {
    return new Response(null, { status: 302, headers: { Location: PRESIGN, "Cache-Control": "no-store" } });
  }

  const asset = await latestAsset();
  if (!asset) return Response.json({ error: "no-artifact" }, { status: 503 });

  if (!GITHUB_TOKEN) {
    return new Response(null, { status: 302, headers: { Location: asset.browser_download_url, "Cache-Control": "no-store" } });
  }

  const res = await fetch(asset.url, {
    headers: { Accept: "application/octet-stream", Authorization: `Bearer ${GITHUB_TOKEN}` },
  });
  if (!res.ok || !res.body) return Response.json({ error: "fetch-failed" }, { status: 502 });

  const headers = new Headers({
    "Content-Type": "application/x-apple-diskimage",
    "Content-Disposition": `attachment; filename="${ASSET}"`,
    "Cache-Control": "no-store",
    "X-Content-Type-Options": "nosniff",
  });
  const length = res.headers.get("content-length");
  if (length) headers.set("Content-Length", length);
  return new Response(res.body, { status: 200, headers });
}
