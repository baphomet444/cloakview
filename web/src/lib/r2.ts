import { presignGet } from "./aws-sigv4.ts";

/**
 * Cloudflare R2 as the artifact store.
 *
 * The bucket stays private; the only way in is a URL signed here, valid for five minutes — long
 * enough for a download to start, short enough that a link pasted into a chat is dead before the
 * next person reads it. Because the signature is minted per request, the env only holds the R2
 * credentials, never a URL that can expire while it sits in a config.
 */

export const DEFAULT_BUCKET = "cloakview-downloads";
export const DEFAULT_KEY = "CloakView-Pro_aarch64.dmg";
const TTL_SECONDS = 5 * 60;

export type R2Config = {
  accountId: string;
  accessKeyId: string;
  secretAccessKey: string;
  bucket: string;
  key: string;
};

type Env = Record<string, string | undefined>;

/** Null unless all three credentials are present, so a half-configured deploy degrades to GitHub. */
export function r2Config(env: Env = process.env): R2Config | null {
  const accountId = env.R2_ACCOUNT_ID?.trim();
  const accessKeyId = env.R2_ACCESS_KEY_ID?.trim();
  const secretAccessKey = env.R2_SECRET_ACCESS_KEY?.trim();
  if (!accountId || !accessKeyId || !secretAccessKey) return null;
  return {
    accountId,
    accessKeyId,
    secretAccessKey,
    bucket: env.R2_BUCKET?.trim() || DEFAULT_BUCKET,
    key: env.R2_KEY?.trim() || DEFAULT_KEY,
  };
}

export function objectUrl(config: R2Config): URL {
  const key = config.key
    .split("/")
    .map((segment) => encodeURIComponent(segment))
    .join("/");
  return new URL(`https://${config.accountId}.r2.cloudflarestorage.com/${config.bucket}/${key}`);
}

export function presignR2Download(config: R2Config, now = new Date()): string {
  return presignGet({
    url: objectUrl(config),
    region: "auto",
    service: "s3",
    accessKeyId: config.accessKeyId,
    secretAccessKey: config.secretAccessKey,
    expiresIn: TTL_SECONDS,
    now,
  }).url;
}
