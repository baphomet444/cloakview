Eres mi desarrollador web. Tu misión es construir el website de marketing de
**CloakView**, una app de escritorio macOS que todavía no tiene sitio web.

CONTEXTO DEL PRODUCTO
Léelo primero (repo ~/cloakview, tienes acceso al disco):
- Spec: `docs/superpowers/specs/2026-11-05-cloakview-design.md`
- Plan: `docs/superpowers/plans/2026-11-05-cloakview-mvp.md`

Resumen: copiloto de reuniones en tiempo real — escucha la llamada del
usuario (Zoom/Meet/Teams), transcribe en vivo, detecta cuándo le toca hablar
y sugiere respuestas en un overlay privado. Diferenciales reales:
- **BYOK-first**: el usuario pone su propia API key de OpenAI; la app habla
  directo con OpenAI. CloakView no cobra suscripción en v1: el usuario solo
  paga su consumo de API a OpenAI.
- **100% local**: sin backend nuestro, sin cuentas, sin telemetría.
- macOS 14.2+, firmada y notarizada por Apple.
- La app es bilingüe (en/es) — el website también lo será.
- Marca verificada: cloakview.app / .ai / .io / .dev libres (cloakview.com
  está en venta, no lo uses aún). GitHub Releases será el canal de
  distribución (repo aún por crear — centraliza la URL en un solo config).

MODO DE TRABAJO
- Por FASES en orden. Al cerrar cada fase: build limpio + reporte de 5 líneas.
  No adelantes fases, no inventes páginas extra.
- DISEÑO ORIGINAL: prohibido copiar diseño, textos, assets o estructura de
  smog-ai.com, cluely, sotto ni ningún competidor. Estándar de landing
  moderna (Linear/Raycast como referencia de calidad, no de contenido).
- Tono de copy: privacidad, local-first, "tu clave, tu control", honestidad
  (sin countdowns falsos, sin "limited offer" inventado).

STACK
- Next.js 15 (App Router) + TypeScript + Tailwind CSS. Deploy en Vercel.
- i18n: inglés por defecto + español en /es, con diccionarios JSON y test de
  paridad de claves (mismo mecanismo que la app). Selector de idioma visible.
- MDX para la guía. sitemap, robots, metadata/OG por página y por idioma.

FASES
1. SCAFFOLD — Next.js + Tailwind + tema (dark elegante, tú eliges paleta y
   la justificas en 1 línea), layout base (nav + footer), i18n en/es con
   paridad testeada, SEO base. Crea `site.config.ts` con TODAS las URLs
   externas centralizadas (GITHUB_RELEASES, GITHUB_REPO, SUPPORT_EMAIL) —
   es el único lugar que edito yo cuando existan.
2. LANDING (/) — hero con mockup SVG original del overlay (ilustración
   propia, no screenshots robados), "cómo funciona" en 3 pasos (conecta tu
   key → escucha → sugiere), grid de features (BYOK, local-first, overlay
   privado, atajos globales, es/en), sección honesta de costos ("pagas tu
   consumo de API, nosotros nada"), FAQ con 8 preguntas de un escéptico
   real, CTA de descarga. Todo el texto en ambos idiomas.
3. /pricing — honesto y simple: CloakView es GRATIS en v1 + estimador de
   costo mensual de API según uso (ligero/moderado/intensivo, con precios
   de OpenAI de referencia marcados como "aprox., según tarifas de OpenAI").
   Nota de roadmap: "Pro de pago único con features avanzadas — visión,
   traducción, modo offline — llegará en v1.1". Nada de planes ficticios.
4. /download — botones con hrefs REALES (nunca JS-only) apuntando a
   GITHUB_RELEASES desde site.config: DMG Apple Silicon, DMG Intel
   ("próximamente"), Windows (waitlist). Cada artefacto muestra su SHA-256
   copiable junto al botón y link "verificar en GitHub". Detección de
   plataforma con fallback manual. Badge "Firmada y notarizada por Apple".
   PROHIBIDO banners de "temporarily unavailable" — si un artefacto no
   existe, no lo muestres.
5. /guide — docs MDX en/es: instalación, permisos de macOS (captura de
   pantalla + mic), cómo crear y conectar la API key de OpenAI (paso a
   paso), atajos, troubleshooting. Índice lateral sticky. Cada paso con su
   versión en español.
6. /faq /support /changelog — FAQ completo (bilingüe), soporte por email +
   GitHub Issues (URLs desde site.config), changelog leído de GitHub
   Releases en build time.
7. LEGAL — /privacy y /terms reales y completos (GDPR/CCPA-aware,
   jurisdicción: Florida, EE.UU.; nombre legal de empresa en constante de
   site.config, claramente marcada), /cookies. Si no hay analytics (es el
   caso), la página de cookies lo dice explícitamente como feature de
   privacidad. Banner de consentimiento solo si algún día hay analytics.
8. HARDENING + DEPLOY — headers: CSP estricto, HSTS, X-Frame-Options,
   referrer-policy. Lighthouse ≥ 95 en performance/a11y/SEO/best-practices
   en ambos idiomas. next/image para todo. Deploy en Vercel, dominio
   cloakview.app configurado.

VERIFICACIÓN FINAL — build de producción limpio en ambos idiomas, sitemap
con /es, cero hrefs rotos (verifica cada uno), Lighthouse 95+ en las 4
métricas.

EMPIEZA por la Fase 1. Al terminar todas, dame: URL de preview de Vercel,
comando de deploy, y la lista exacta de valores que debo poner en
site.config.ts (URL del repo GitHub, email de soporte, nombre legal).
