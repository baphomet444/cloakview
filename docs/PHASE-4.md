# Fase 4 — /download

Lee docs/BRIEF-WEB.md para el contexto del producto.

## Contexto
- site.config.ts centraliza URLs. isGithubPending() devuelve true (placeholder OWNER).
- DownloadButton ya existe y respeta isGithubPending(). Reutilízalo.
- Paleta y patrones de grid ya establecidos. No dupliques estilos.

## Requisitos
1. Ruta /download bilingüe (/en/download + /es/download), prerenderizada,
   con metadata/OG por idioma y entrada en sitemap.
2. Tres artefactos: DMG Apple Silicon, DMG Intel, Windows.
   - Apple Silicon: botón principal, activo cuando el repo exista.
   - Intel: "próximamente".
   - Windows: "no disponible en v1". NO waitlist (contradice "sin backend").
3. Mientras isGithubPending() sea true: botones deshabilitados con texto
   "próximamente". PROHIBIDO el banner "temporarily unavailable".
   PROHIBIDO cualquier href="#" o enlace muerto.
4. Cada artefacto muestra su SHA-256 copiable junto al botón y un enlace
   "verificar en GitHub". Cuando no haya hash disponible, muestra el bloque
   SHA en estado pendiente (texto tenue, sin botón de copiar activo).
5. El hash NO debe estar hardcodeado en el componente: debe leerse de una
   constante en site.config.ts (ej. SHA256_ARM64), para que se edite en un
   solo sitio.
6. Detección de plataforma con fallback manual: si detecta macOS ARM,
   destaca Apple Silicon; si no puede detectar, muestra las tres opciones
   sin destacar ninguna. Nunca ocultes opciones por detección.
7. Badge "Firmada y notarizada por Apple" junto al artefacto macOS.
8. Todo el copy en en.json y es.json. Cero strings hardcodeados.
9. Accesibilidad: los botones deshabilitados usan aria-disabled, no
   pointer-events:none a secas. El SHA copiable tiene botón con
   aria-label descriptivo y feedback accesible al copiar.

## Cierre
`npm run build` limpio, `npm run test:i18n` en verde, reporte de 5 líneas.
No empieces la Fase 5.
