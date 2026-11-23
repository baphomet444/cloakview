#!/usr/bin/env python3
"""Upload a file to the private R2 bucket that /api/dl signs URLs for.

Cloudflare's R2 API tokens are S3 credentials: the S3 API is the only way to PUT an object
without pulling a multi-megabyte SDK into the web app, and the same keys the site uses to
presign downloads can also publish the artifact. Signing is done here by hand (SigV4,
query-free header auth) so this needs nothing but the Python standard library.

Credentials come from the environment, or from ~/.cloakview-r2.env:

    R2_ACCOUNT_ID=...
    R2_ACCESS_KEY_ID=...
    R2_SECRET_ACCESS_KEY=...
    R2_BUCKET=cloakview-downloads        # optional
    R2_KEY=CloakView-Pro_aarch64.dmg     # optional

Usage: python3 scripts/r2-put.py <file> [key]
"""
import datetime
import hashlib
import hmac
import http.client
import os
import sys

ENV_FILE = os.path.expanduser("~/.cloakview-r2.env")
DEFAULT_BUCKET = "cloakview-downloads"
DEFAULT_KEY = "CloakView-Pro_aarch64.dmg"


def load_env():
    env = dict(os.environ)
    if os.path.exists(ENV_FILE):
        with open(ENV_FILE, encoding="utf-8") as handle:
            for line in handle:
                line = line.strip()
                if not line or line.startswith("#") or "=" not in line:
                    continue
                name, value = line.split("=", 1)
                env.setdefault(name.strip(), value.strip())
    return env


def sha256_hex(data):
    return hashlib.sha256(data).hexdigest()


def sign(key, message):
    return hmac.new(key, message.encode("utf-8"), hashlib.sha256).digest()


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)

    env = load_env()
    missing = [k for k in ("R2_ACCOUNT_ID", "R2_ACCESS_KEY_ID", "R2_SECRET_ACCESS_KEY") if not env.get(k)]
    if missing:
        sys.exit(f"missing R2 credentials: {', '.join(missing)} (see {ENV_FILE})")

    account_id = env["R2_ACCOUNT_ID"]
    bucket = env.get("R2_BUCKET") or DEFAULT_BUCKET
    key = sys.argv[2] if len(sys.argv) > 2 else (env.get("R2_KEY") or DEFAULT_KEY)
    source = sys.argv[1]
    with open(source, "rb") as handle:
        payload = handle.read()

    payload_hash = sha256_hex(payload)
    now = datetime.datetime.now(datetime.timezone.utc)
    amz_date = now.strftime("%Y%m%dT%H%M%SZ")
    date_stamp = now.strftime("%Y%m%d")

    host = f"{account_id}.r2.cloudflarestorage.com"
    uri = f"/{bucket}/{key}"
    headers = (
        f"host:{host}\n"
        f"x-amz-content-sha256:{payload_hash}\n"
        f"x-amz-date:{amz_date}\n"
    )
    signed_headers = "host;x-amz-content-sha256;x-amz-date"
    canonical_request = f"PUT\n{uri}\n\n{headers}\n{signed_headers}\n{payload_hash}"

    scope = f"{date_stamp}/auto/s3/aws4_request"
    to_sign = f"AWS4-HMAC-SHA256\n{amz_date}\n{scope}\n{sha256_hex(canonical_request.encode())}"

    k_date = sign(("AWS4" + env["R2_SECRET_ACCESS_KEY"]).encode(), date_stamp)
    k_region = sign(k_date, "auto")
    k_service = sign(k_region, "s3")
    k_signing = sign(k_service, "aws4_request")
    signature = hmac.new(k_signing, to_sign.encode(), hashlib.sha256).hexdigest()

    authorization = (
        f"AWS4-HMAC-SHA256 Credential={env['R2_ACCESS_KEY_ID']}/{scope}, "
        f"SignedHeaders={signed_headers}, Signature={signature}"
    )

    connection = http.client.HTTPSConnection(host, timeout=300)
    connection.request(
        "PUT",
        uri,
        body=payload,
        headers={
            "x-amz-content-sha256": payload_hash,
            "x-amz-date": amz_date,
            "Authorization": authorization,
            "Content-Type": "application/x-apple-diskimage",
            "Content-Length": str(len(payload)),
        },
    )
    response = connection.getresponse()
    body = response.read().decode("utf-8", "replace")

    print(f"PUT {bucket}/{key} -> {response.status} {response.reason}")
    print(f"bytes:  {len(payload)}")
    print(f"sha256: {payload_hash}")
    print(f"etag:   {response.getheader('ETag')}")
    if response.status >= 300:
        print(body[:800])
        sys.exit(1)


if __name__ == "__main__":
    main()
