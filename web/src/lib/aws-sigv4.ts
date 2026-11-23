import { createHash, createHmac } from "node:crypto";

/**
 * AWS Signature Version 4, query-string flavour — enough to presign a GET and nothing else.
 *
 * Written by hand instead of pulling in @aws-sdk/client-s3: the only thing this site needs is
 * "give me a URL that expires", and the SDK adds megabytes to a route that runs on every click.
 * The algorithm is pinned by AWS's published example (see tests/aws-sigv4.test.ts), so a wrong
 * implementation fails the suite instead of failing at the CDN.
 */

const ALGORITHM = "AWS4-HMAC-SHA256";
const UNSIGNED = "UNSIGNED-PAYLOAD";

const sha256Hex = (input: string): string => createHash("sha256").update(input, "utf8").digest("hex");

const hmac = (key: string | Buffer, data: string): Buffer =>
  createHmac("sha256", key).update(data, "utf8").digest();

/** RFC 3986: everything except unreserved characters is percent-encoded, "/" only when asked. */
export function uriEncode(value: string, encodeSlash = true): string {
  return encodeURIComponent(value)
    .replace(/[!'()*]/g, (char) => `%${char.charCodeAt(0).toString(16).toUpperCase()}`)
    .replace(/%2F/g, encodeSlash ? "%2F" : "/");
}

/** `/bucket/a b.dmg` -> `/bucket/a%20b.dmg`; the slashes stay literal (S3 wants them that way).
 *  Segments arrive percent-encoded from `URL.pathname`, so they are decoded first: encoding a
 *  `%20` again would sign `%2520` and the CDN would reject the request. */
function canonicalUri(pathname: string): string {
  const path = pathname.startsWith("/") ? pathname : `/${pathname}`;
  return path
    .split("/")
    .map((segment) => {
      let raw = segment;
      try {
        raw = decodeURIComponent(segment);
      } catch {
        // A stray "%" in the key: keep it verbatim and let uriEncode escape it.
      }
      return uriEncode(raw, false);
    })
    .join("/");
}

export type PresignOptions = {
  method?: string;
  url: URL;
  region: string;
  service: string;
  accessKeyId: string;
  secretAccessKey: string;
  expiresIn: number;
  now?: Date;
};

export type PresignedRequest = {
  url: string;
  signature: string;
  canonicalRequest: string;
  stringToSign: string;
};

/** Full result, so tests can assert on the intermediate strings AWS documents. */
export function presignRequest(options: PresignOptions): PresignedRequest {
  const method = (options.method ?? "GET").toUpperCase();
  const now = options.now ?? new Date();
  const amzDate = now.toISOString().replace(/[:-]|\.\d{3}/g, "");
  const dateStamp = amzDate.slice(0, 8);
  const scope = `${dateStamp}/${options.region}/${options.service}/aws4_request`;

  const params: Record<string, string> = {};
  for (const [key, value] of options.url.searchParams) params[key] = value;
  params["X-Amz-Algorithm"] = ALGORITHM;
  params["X-Amz-Credential"] = `${options.accessKeyId}/${scope}`;
  params["X-Amz-Date"] = amzDate;
  params["X-Amz-Expires"] = String(options.expiresIn);
  params["X-Amz-SignedHeaders"] = "host";

  const canonicalQuery = Object.keys(params)
    .sort()
    .map((key) => `${uriEncode(key)}=${uriEncode(params[key] ?? "")}`)
    .join("&");

  const path = canonicalUri(options.url.pathname);
  const canonicalHeaders = `host:${options.url.host}\n`;
  const canonicalRequest = [
    method,
    path,
    canonicalQuery,
    canonicalHeaders,
    "host",
    UNSIGNED,
  ].join("\n");
  const stringToSign = [ALGORITHM, amzDate, scope, sha256Hex(canonicalRequest)].join("\n");

  const signingKey = hmac(
    hmac(hmac(hmac(`AWS4${options.secretAccessKey}`, dateStamp), options.region), options.service),
    "aws4_request",
  );
  const signature = createHmac("sha256", signingKey).update(stringToSign, "utf8").digest("hex");

  return {
    url: `${options.url.origin}${path}?${canonicalQuery}&X-Amz-Signature=${signature}`,
    signature,
    canonicalRequest,
    stringToSign,
  };
}

export const presignGet = (options: Omit<PresignOptions, "method">): PresignedRequest =>
  presignRequest({ ...options, method: "GET" });
