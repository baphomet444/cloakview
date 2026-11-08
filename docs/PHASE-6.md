# Fase 6 — /faq, /support, /changelog

Lee docs/BRIEF-WEB.md para el contexto del producto.

## Contexto
- Repo GitHub NO existe (site.config.ts con OWNER). isGithubPending() = true.
- Next 16.4.5, React 19.3, Tailwind 4. params es Promise.
- Paleta, componentes y diccionarios establecidos en fases 1-5.

## /faq
1. Bilingüe, mínimo 15 preguntas por tema: privacidad, instalación, costes,
   compatibilidad, técnico.
2. Acordeón accesible: button con aria-expanded y aria-controls, panel con id,
   navegable por teclado, foco visible.

## /support
3. Email desde siteConfig.SUPPORT_EMAIL (mailto).
4. Enlace a GitHub Issues desde siteConfig.GITHUB_REPO. Con isGithubPending()
   true: "próximamente" sin href muerto.
5. Sección "qué incluir al reportar un bug": versión macOS, versión app, pasos.
   Y qué NO incluir: claves de API, audio.
6. Enlace a la guía de troubleshooting.

## /changelog
7. Bilingüe. Lee GitHub Releases en build time CON FALLBACK obligatorio a
   src/content/changelog.json.
   - Si isGithubPending(): no intentes el fetch, ve directo al fallback.
   - fetch con AbortSignal.timeout(3000).
   - El build NUNCA debe romperse por esto.
   - Marca visualmente cuando el contenido viene del fallback local.
8. JSON de changelog bilingüe, paridad de claves testeada.

## General
9. Todo el copy9.e UI en en.json/es.json. Cero strings hardcodeados.
10. Metadata/OG por página e idioma. Sitemap con las tres rutas en ambos idiomas.
11. Enlaces externos con rel="noopener noreferrer".

## Cierre
npm run build limpio CON el repo inexistente, npm run test:i18n en verde,
npm run lint limpio, reporte de 5 líneas. No empieces la Fase 7.
