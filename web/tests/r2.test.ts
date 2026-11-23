import { test } from "node:test";
import assert from "node:assert/strict";
import { presignGet } from "../src/lib/aws-sigv4.ts";
import { presignR2Download, objectUrl, r2Config, DEFAULT_KEY } from "../src/lib/r2.ts";

/**
 * Pinned to the example AWS publishes for "Authenticating Requests: Using Query Parameters
 * (AWS Signature Version 4)". If signing drifts, this fails here instead of turning into an
 * opaque AccessDenied at the storage edge.
 */
test("la firma coincide con el ejemplo publicado por AWS", () => {
  const signed = presignGet({
    url: new URL("https://examplebucket.s3.amazonaws.com/test.txt"),
    region: "us-east-1",
    service: "s3",
    accessKeyId: "AKIAIOSFODNN7EXAMPLE",
    secretAccessKey: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
    expiresIn: 86400,
    now: new Date("2013-05-24T00:00:00Z"),
  });
  assert.equal(
    signed.signature,
    "aeeed9bbccd4d02ee5c0109b86d86835f995330da4c265957d157751f604d404",
  );
  assert.match(signed.stringToSign, /^AWS4-HMAC-SHA256\n20130524T000000Z\n20130524\/us-east-1\/s3\/aws4_request\n/);
});

test("un objeto con espacio en el nombre se firma una sola vez codificado", () => {
  const signed = presignGet({
    url: new URL("https://acc.r2.cloudflarestorage.com/bucket/CloakView Pro.dmg"),
    region: "auto",
    service: "s3",
    accessKeyId: "key",
    secretAccessKey: "secret",
    expiresIn: 300,
    now: new Date("2026-01-01T00:00:00Z"),
  });
  assert.match(signed.url, /\/bucket\/CloakView%20Pro\.dmg\?/);
  assert.doesNotMatch(signed.url, /%2520/);
});

test("la URL prefirmada de R2 usa el endpoint de la cuenta, expira y lleva firma", () => {
  const config = {
    accountId: "abc123",
    accessKeyId: "key",
    secretAccessKey: "secret",
    bucket: "cloakview-downloads",
    key: DEFAULT_KEY,
  };
  const url = new URL(presignR2Download(config, new Date("2026-01-01T00:00:00Z")));
  assert.equal(url.host, "abc123.r2.cloudflarestorage.com");
  assert.equal(url.pathname, `/cloakview-downloads/${DEFAULT_KEY}`);
  assert.equal(url.searchParams.get("X-Amz-Expires"), "300");
  assert.equal(url.searchParams.get("X-Amz-Algorithm"), "AWS4-HMAC-SHA256");
  assert.match(url.searchParams.get("X-Amz-Credential") ?? "", /^key\/20260101\/auto\/s3\/aws4_request$/);
  assert.match(url.searchParams.get("X-Amz-Signature") ?? "", /^[0-9a-f]{64}$/);
});

test("sin credenciales completas R2 queda desactivado", () => {
  assert.equal(r2Config({}), null);
  assert.equal(r2Config({ R2_ACCOUNT_ID: "abc" }), null);
  assert.equal(
    r2Config({ R2_ACCOUNT_ID: "abc", R2_ACCESS_KEY_ID: "k", R2_SECRET_ACCESS_KEY: "s" })?.bucket,
    "cloakview-downloads",
  );
});

test("objectUrl codifica cada segmento de la clave", () => {
  const url = objectUrl({
    accountId: "acc",
    accessKeyId: "k",
    secretAccessKey: "s",
    bucket: "b",
    key: "nested/CloakView.dmg",
  });
  assert.equal(url.pathname, "/b/nested/CloakView.dmg");
});
