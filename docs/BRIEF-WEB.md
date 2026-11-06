# CloakView — Website de marketing

Eres mi desarrollador web. Misión: construir el website de marketing de CloakView,
una app de escritorio macOS que todavía no tiene sitio web.

## CONTEXTO DEL PRODUCTO

Lee primero (repo ~/cloakview, tienes acceso al disco):
- Spec: docs/superpowers/specs/2026-11-05-cloakview-design.md
- Plan: docs/superpowers/plans/2026-11-05-cloakview-mvp.md

Resumen: copiloto de reuniones en tiempo real — escucha la llamada del usuario
(Zoom/Meet/Teams), transcribe en vivo, detecta cuándo le toca hablar y sugiere
respuestas en un overlay privado. Diferenciales reales:
- BYOK-first: el usuario pone su propia API key de OpenAI; la app habla directo
  con OpenAI. Sin suscripción en v1: solo paga su consumo de API.
- 100% local: sin backend nuestro, sin cuentas, sin telemetría.
- macOS 14.2+, firmada y notarizada por Apple.
- App bilingüe (en/es); el website también.
- Marca: cloakview.app / .ai / .io / .dev libres (cloakview.com NO usar).
  GitHub Releases será el canal de distribución (repo por crear).

## MODO DE TRABAJO

- Por FASES en orden. Al cerrar cada fase: build limpio + reporte de 5 líneas.
- No adelantes fases, no inventes páginas extra.
- DISEÑO ORIGINAL: prohibido copiar diseño, textos, assets o estructura de
  smog-ai.com, cluely, sotto ni ningún competidor. Estándar de landing  smog-ai.com, cluely, sotto ni ningúncia de calidad, no de contenido).
- Tono: privacidad, local-first, "tu clave, tu control", honestidad.
  Sin countdowns falsos, sin "limited offer" inventado.

## STACK

- Next.js 15 (App Router) + TypeScript + Tailwind CSS. Deploy en Vercel.
- i18n: inglés por defecto + español en /es, diccionarios JSON y test de
- i18n: inglés por defecto + español en /es, diccionarios JSON ía. sitemap, robots, metadata/OG por página y por idioma.

## FASES

1. SCAFFOLD — Next.js + Tailwind + tema (dark elegante, elige paleta y
   justifícala en 1 línea), layout base (nav + footer), i18n en/es con paridad
   testeada, SEO base. Crear site.config.ts con TODAS las URLs externas
   centralizadas (GITHUB_RELEASES, GITHUB_REPO, SUPPORT_EMAIL).

2. LANDING (/) — hero con mockup SVG original del overlay (ilustración propia,
   no scr   no scr   no scr   no scr   no scr   no scr   no scr   no scr   no scr   ug   no scr   no scr   no scr   no scr   no scr   no scr   no scr   no scr   no scr   ug   no scr   no scr   no scr   no scr   no scr   no scr ,
   nosotros nada"). FAQ con 8 preguntas de un escéptico real. CTA de descarga.
   Todo en ambos idiomas.

3. /pricing — CloakView es GRATIS en v1 + estimador de costo mensual de API
   según uso (ligero/moderado/intensivo, con precios de OpenAI marcados como
   "aprox., según tarifas de OpenAI"). Nota de roadmap: "Pro de pago único con
   features avanzadas — visión, traducción, modo offline — llegará en v   features avanzadas — visión, traducción, modo offline — llegs REALES (nunca JS-only) apuntando a
   GITHUB_RELEASES desde site.config: DMG Apple Silicon, DMG Intel
   ("próximamente"), Windows (no disponible en v1). Cada artefacto muestra su
   SHA-256 copiable junto al botón y link "verificar en GitHub". Detección de
   plataforma con fallback manual. Badge    plataforma con fallback manual. Badge    plataforma con fallback manual. Badge    plataforma con fallback manual. Badge    plataforma con fallback manual. Badge    plataforma con fallback manual. Badge    plataforma con fal), cómo crear y conectar la API key de OpenAI (paso a paso), atajos,
   troubleshooting. Índice lateral sticky. Cada paso en ambos idiomas.

6. /faq /support /changelog — FAQ completo bilingüe, soporte por email +
   GitHub Issues (URLs desde site.config), changelog leído de GitHub Releases
   en build time.

7. LEGAL — /privacy y /terms reales y completos (GDPR/CCPA-aware, jurisdicción
   Florida, EE.UU.; nombre legal en constante de site.config claramente marcada),
   /cookies. Sin analytics: la página de cookies lo dice explícitamente como
   feature de privacidad. Banner de consentimiento solo si algún día hay analytics.

8. HARDENING + DEPLOY — headers: CSP estricto, HSTS, X-Frame-Options,
   referrer-policy. Lighthouse >= 95 en performance/a11y/SEO/best-practices en
   ambos idiomas. next/image para todo. Deploy en Vercel, dominio
   cloakview.app configurado.

## VERIFICACIÓN FINAL

Build de producción limpio en ambos idiomas, sitemap con /es, cero hrefs rotos
(verifica cada uno), Lighthouse 95+ en las 4 métricas.

Al terminar todas, entregar: URL de preview de Vercel, comando de deploy, y la
lista exacta de valores pendientlista exacta de valores pendientlista exacta de valores pendientlista exantes de empezar, verifica el repo: `ls ~/cloakview`. Si no existe, créalo y
  pide confirmación ant  pide confirmación ant  pide confirmación ant  pide confirmación ant  pide--chrome-flags="--headless"`
  por terminal. NO usar herramientas de navegador para esto.
- Vercel: VERCEL_TOKEN debe existir en .env antes de la Fase 8. Si no está,
  detente y pide el token. No intentes desplegar sin él.
- No ejecutes `npm install` de paquetes no listados en el stack sin justificarlo.

## VALORES DE site.config.ts (placeholders hasta que existan)

GITHUB_RELEASES = "https://github.com/OWNER/cloakview/releases/latest"
GITHUB_REPO     = "https://GITHUB_REPO     = "https://GSUPPORT_EMAIL   = "soporte@cloakview.app"
LEGAL_ENTITY    = "PLACEHOLDER LLC"   # marcar visiblemente como pendiente

Regla: si GITHUB_RELEASES contiene "OWNER", laRegla: si GITHUB_RELEASES contiene "OWNER", laRegla: si GITHUB_RELEASES cont, no un banner de error.
