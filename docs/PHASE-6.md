# Fase 6 — /faq, /support, /changelog

Lee docs/BRIEF-WEB.md para el contexto del producto.

## Contexto
- Repo de GitHub NO existe (site.config.ts con OWNER). isGithubPending() = true.
- Paleta, componentes y diccionarios ya establecidos en fases 1-5.
- Next 16.4.5, React 19.3, Tailwind 4 (@theme en CSS). params es Promise.

## Requisitos

### /faq
1. Bilingüe, mínimo 15 preguntas agrupadas por tema: privacidad, instalación,
   costes, compatibilidad, técnico.
2. Acordeón accesible: <button> con aria-expanded y aria-controls apuntando
   al panel, panel con id correspondiente. Navegable con teclado, foco visible.
   NO uses <details> sin estilos ni divs clicables.
3. Cada respuesta puede enlazar a un artículo de la guía cuando exista.

### /support
4. Bilingüe. Email desde siteConfig.SUPPORT_EMAIL como enlace mailto.
5. Enlace a GitHub Issues desde siteConfig.GITHUB_REPO. Mientras
   isGithubPending() sea true, muestra "próximamente" sin href muerto.
6. Sección "qué incluir al reportar un bug": versión de macOS, versión de la
   app, pasos para reproducir, y qué NO incluir (claves de API, audio).
7. Enlace a la guía de troubleshooting.

### /changelog
8. Bilingüe. Lee GitHub Releases en build time CON FALLBACK obligatorio a
   src/content/c   src/content/c   src/content/c   src/content/c   src/content/c   src/ch falla, 404, o el repo no existe: usa el JSON local. El build
     NUNCA debe romperse por esto.
   - Si el   - Si el   - Si el   - Si el   - no intentes el fetch siquiera:
     ve directo al fallback.
   - Marca visualmente cuando el contenido viene del fallback local.
9. El JSON de changelog es bilingüe: cada entrada con title/body en en y es.
   Paridad de claves testeada.

## General
10. Todo el copy de UI en en.json/es.json. Cero strings hardcodeados.
11. Metadata/OG por página y por idioma. Sitemap actualizado con las tres rutas
    en ambos idiomas.
12. Enlaces externos con rel="noopener noreferrer".

## Cierre
`npm run build` limpio CON el repo inexistente (esto prueba el fallback),
`npm run test:i18n` en verde, `npm run lint` limpio, reporte de 5 líneas.
No empieces la Fase 7.
