import "server-only";
import localEntries from "@/content/changelog.json";
import { GITHUB_API } from "@/site.config";
import { isGithubPending, siteConfig, type Locale } from "@/site.config";

type Localized = Record<Locale, string>;
type Entry = { version: string; date: string | null; title: Localized; body: Localized };
type Release = { tag_name?: unknown; name?: unknown; body?: unknown; published_at?: unknown; html_url?: unknown; draft?: unknown; prerelease?: unknown };

const fallback = localEntries as Entry[];

function isLocalized(value: unknown): value is Localized {
  return typeof value === "object" && value !== null && siteConfig.locales.every((locale) => typeof (value as Record<string, unknown>)[locale] === "string" && Boolean((value as Record<string, string>)[locale].trim()));
}

export function validateChangelog(entries: unknown): entries is Entry[] {
  return Array.isArray(entries) && entries.length > 0 && entries.every((entry) =>
    typeof entry === "object" && entry !== null &&
    typeof entry.version === "string" && Boolean(entry.version.trim()) &&
    (entry.date === null || (typeof entry.date === "string" && /^\d{4}-\d{2}-\d{2}$/.test(entry.date) && !Number.isNaN(Date.parse(entry.date)))) &&
    isLocalized(entry.title) && isLocalized(entry.body));
}

export async function getChangelog(): Promise<{ entries: Entry[]; isFallback: boolean }> {
  "use cache";
  if (!validateChangelog(fallback)) throw new Error("Invalid local changelog");
  // Do not make a request when the repository is still a placeholder.
  if (isGithubPending()) return { entries: fallback, isFallback: true };
  try {
    const repo = new URL(siteConfig.GITHUB_REPO);
    const match = /^\/([^/]+)\/([^/]+)\/?$/.exec(repo.pathname);
    if (repo.hostname !== "github.com" || !match) return { entries: fallback, isFallback: true };
    const response = await fetch(`${GITHUB_API}/repos/${match[1]}/${match[2]}/releases`, {
      headers: { Accept: "application/vnd.github+json" },
      signal: AbortSignal.timeout(3000),
    });
    if (!response.ok) return { entries: fallback, isFallback: true };
    const releases: unknown = await response.json();
    if (!Array.isArray(releases)) return { entries: fallback, isFallback: true };
    // Public GitHub release notes have no guaranteed translation. Only use releases
    // containing an explicit bilingual JSON payload; never label English as Spanish.
    const entries: unknown = releases.filter((release: Release) => release.draft !== true && release.prerelease !== true).map((release: Release) => {
      // One malformed release must not hide other valid bilingual releases.
      let payload: unknown = null;
      if (typeof release.body === "string") {
        try { payload = JSON.parse(release.body); } catch { /* no bilingual payload */ }
      }
      return { version: release.tag_name, date: typeof release.published_at === "string" ? release.published_at.slice(0, 10) : null, title: (payload as Record<string, unknown> | null)?.title, body: (payload as Record<string, unknown> | null)?.body };
    });
    return validateChangelog(entries) ? { entries, isFallback: false } : { entries: fallback, isFallback: true };
  } catch {
    return { entries: fallback, isFallback: true };
  }
}
