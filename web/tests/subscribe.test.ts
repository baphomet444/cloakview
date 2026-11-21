// El endpoint de captura de email: 503 sin destino configurado, 400 con un email
// inválido y un POST JSON transparente cuando NEWSLETTER_ENDPOINT existe.
// Sin red: `fetch` se reemplaza por un espía.

import { test } from "node:test";
import assert from "node:assert/strict";
import { POST } from "../src/app/api/subscribe/route.ts";

function request(body: unknown, ip = "203.0.113.1"): Request {
  return new Request("https://getcloakview.com/api/subscribe", {
    method: "POST",
    headers: { "Content-Type": "application/json", "x-forwarded-for": ip },
    body: JSON.stringify(body),
  });
}

type Call = { url: string; init: RequestInit };

function stubFetch(response: Response) {
  const calls: Call[] = [];
  const original = globalThis.fetch;
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    calls.push({ url: String(input), init: init ?? {} });
    return response;
  }) as typeof fetch;
  return { calls, restore: () => void (globalThis.fetch = original) };
}

function clearEnv() {
  delete process.env.NEWSLETTER_ENDPOINT;
  delete process.env.NEWSLETTER_API_KEY;
  delete process.env.NEWSLETTER_EMAIL_FIELD;
}

test("sin NEWSLETTER_ENDPOINT responde 503 y no rompe nada", async () => {
  clearEnv();
  const response = await POST(request({ email: "someone@example.com" }));
  assert.equal(response.status, 503);
  const body = (await response.json()) as { error: string; message: string };
  assert.equal(body.error, "not-configured");
  assert.match(body.message, /NEWSLETTER_ENDPOINT/);
  assert.equal(response.headers.get("cache-control"), "no-store");
});

test("un email inválido no se reenvía al proveedor", async () => {
  clearEnv();
  process.env.NEWSLETTER_ENDPOINT = "https://api.example.test/subscribers";
  const spy = stubFetch(Response.json({ ok: true }));
  try {
    for (const email of ["", "nope", "a@b", "a b@example.com", 42]) {
      const response = await POST(request({ email }, "203.0.113.9"));
      assert.equal(response.status, 400);
      assert.equal(((await response.json()) as { error: string }).error, "invalid-email");
    }
    assert.equal(spy.calls.length, 0);
  } finally {
    spy.restore();
    clearEnv();
  }
});

test("reenvía el email al proveedor como POST JSON y normaliza el caso", async () => {
  clearEnv();
  process.env.NEWSLETTER_ENDPOINT = "https://api.example.test/subscribers";
  process.env.NEWSLETTER_API_KEY = "test-key";
  const spy = stubFetch(Response.json({ ok: true }, { status: 201 }));
  try {
    const response = await POST(request({ email: "  Someone@Example.COM ", locale: "es" }));
    assert.equal(response.status, 200);
    assert.deepEqual(await response.json(), { ok: true });
    assert.equal(spy.calls.length, 1);
    assert.equal(spy.calls[0].url, "https://api.example.test/subscribers");
    assert.equal(spy.calls[0].init.method, "POST");
    const headers = spy.calls[0].init.headers as Record<string, string>;
    assert.equal(headers.Authorization, "Bearer test-key");
    assert.deepEqual(JSON.parse(String(spy.calls[0].init.body)), {
      email: "someone@example.com",
      locale: "es",
      source: "getcloakview.com",
    });
  } finally {
    spy.restore();
    clearEnv();
  }
});

test("NEWSLETTER_EMAIL_FIELD cambia el nombre del campo (Buttondown)", async () => {
  clearEnv();
  process.env.NEWSLETTER_ENDPOINT = "https://api.buttondown.test/v1/subscribers";
  process.env.NEWSLETTER_EMAIL_FIELD = "email_address";
  const spy = stubFetch(Response.json({ ok: true }));
  try {
    const response = await POST(request({ email: "one@example.com" }, "203.0.113.20"));
    assert.equal(response.status, 200);
    const sent = JSON.parse(String(spy.calls[0].init.body)) as Record<string, string>;
    assert.equal(sent.email_address, "one@example.com");
    assert.equal(sent.email, undefined);
  } finally {
    spy.restore();
    clearEnv();
  }
});

test("un error del proveedor sale como 502, no como éxito", async () => {
  clearEnv();
  process.env.NEWSLETTER_ENDPOINT = "https://api.example.test/subscribers";
  const spy = stubFetch(new Response("boom", { status: 500 }));
  try {
    const response = await POST(request({ email: "two@example.com" }, "203.0.113.30"));
    assert.equal(response.status, 502);
    assert.equal(((await response.json()) as { error: string }).error, "upstream-error");
  } finally {
    spy.restore();
    clearEnv();
  }
});
