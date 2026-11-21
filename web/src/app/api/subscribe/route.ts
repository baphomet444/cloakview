/**
 * Captura de email del sitio: recibe `{ email, locale }` y lo reenvía tal cual, como
 * JSON, al endpoint del proveedor configurado en `NEWSLETTER_ENDPOINT`
 * (Buttondown, Resend, ConvertKit o cualquier webhook que acepte un POST JSON).
 *
 * El destino vive en una env var para que cambiar de proveedor —o apagar la lista—
 * no requiera un deploy. Sin la variable el endpoint responde 503 con un mensaje
 * claro: el formulario lo muestra y el resto del sitio sigue funcionando.
 * El build nunca depende de esta variable.
 *
 * Sin imports a propósito: `tests/subscribe.test.ts` importa este módulo con node:test.
 */

const DESTINATION_ENV = "NEWSLETTER_ENDPOINT";
const KEY_ENV = "NEWSLETTER_API_KEY";
const FIELD_ENV = "NEWSLETTER_EMAIL_FIELD";

/** Deliberadamente laxo: la validación de verdad la hace el proveedor al enviar el correo de confirmación. */
const EMAIL = /^[^\s@]+@[^\s@]+\.[^\s@]{2,}$/;

const NO_STORE = { "Cache-Control": "no-store" } as const;

/** Primer filtro contra un script que llene la lista: por instancia, por IP. */
const WINDOW_MS = 10 * 60 * 1000;
const MAX_PER_WINDOW = 6;
const hits = new Map<string, { start: number; count: number }>();

function allow(ip: string): boolean {
  const now = Date.now();
  const entry = hits.get(ip);
  if (!entry || now - entry.start > WINDOW_MS) {
    hits.set(ip, { start: now, count: 1 });
    return true;
  }
  entry.count += 1;
  return entry.count <= MAX_PER_WINDOW;
}

function json(body: unknown, status: number): Response {
  return Response.json(body, { status, headers: NO_STORE });
}

export async function POST(request: Request): Promise<Response> {
  const endpoint = process.env[DESTINATION_ENV]?.trim();
  if (!endpoint) {
    // No es un error de código: la lista todavía no está conectada.
    return json(
      {
        error: "not-configured",
        message: `Newsletter signup is not configured: set ${DESTINATION_ENV} in the deployment environment.`,
      },
      503,
    );
  }

  const ip = request.headers.get("x-forwarded-for")?.split(",")[0]?.trim() || "unknown";
  if (!allow(ip)) return json({ error: "rate-limited" }, 429);

  let body: unknown;
  try {
    body = await request.json();
  } catch {
    return json({ error: "bad-request" }, 400);
  }

  const raw = typeof body === "object" && body !== null ? (body as { email?: unknown }) : {};
  const email = typeof raw.email === "string" ? raw.email.trim().toLowerCase() : "";
  if (!EMAIL.test(email) || email.length > 254) return json({ error: "invalid-email" }, 400);
  const locale = (body as { locale?: unknown }).locale === "es" ? "es" : "en";

  // El nombre del campo cambia por proveedor (Buttondown usa `email_address`); el
  // resto del payload es el mismo para todos.
  const field = process.env[FIELD_ENV]?.trim() || "email";
  const payload: Record<string, unknown> = {
    [field]: email,
    locale,
    source: "getcloakview.com",
  };

  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    Accept: "application/json",
  };
  const key = process.env[KEY_ENV]?.trim();
  if (key) headers.Authorization = `Bearer ${key}`;

  let upstream: Response;
  try {
    upstream = await fetch(endpoint, {
      method: "POST",
      headers,
      body: JSON.stringify(payload),
      signal: AbortSignal.timeout(8000),
    });
  } catch {
    console.error("[subscribe] provider unreachable");
    return json({ error: "upstream-unreachable" }, 502);
  }

  if (!upstream.ok) {
    // El cuerpo del proveedor puede traer detalles útiles en el log, nunca al visitante.
    console.error(`[subscribe] provider responded ${upstream.status}`);
    return json({ error: "upstream-error" }, 502);
  }

  return json({ ok: true }, 200);
}
