import { createHmac, timingSafeEqual } from "node:crypto";

/**
 * Download tokens: 15 minutes is long enough to survive a slow click and short enough that
 * a URL pasted into a chat dies before it is useful.
 */
const TTL_MS = 15 * 60 * 1000;

function secret(): string {
  const value = process.env.DL_SECRET;
  if (!value || value.length < 32) throw new Error("DL_SECRET missing or shorter than 32 chars");
  return value;
}

const sign = (payload: string) => createHmac("sha256", secret()).update(payload).digest("base64url");

export function mint(now = Date.now()): string {
  const payload = String(now + TTL_MS);
  return `${payload}.${sign(payload)}`;
}

export function verify(token: string | null, now = Date.now()): boolean {
  if (!token) return false;
  const dot = token.lastIndexOf(".");
  if (dot < 1) return false;
  const payload = token.slice(0, dot);
  const given = Buffer.from(token.slice(dot + 1));
  const want = Buffer.from(sign(payload));
  if (given.length !== want.length || !timingSafeEqual(given, want)) return false;
  const expiry = Number(payload);
  return Number.isFinite(expiry) && expiry > now;
}
