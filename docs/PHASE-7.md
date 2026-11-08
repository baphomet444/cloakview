# Fase 7 — LEGAL: /privacy, /terms, /cookies

Lee docs/BRIEF-WEB.md, sección FASES punto 7.

## Requisitos
1. /privacy y /terms bilingües, completos, redactados para ESTE producto.
2. Hechos que deben aparecer, porque son ciertos y diferenciales:
   - La app no tiene backend propio ni cuentas.
   - El audio se envía a la API de OpenAI con la clave del usuario.
   - No hay telemetría ni analytics.
   - El usuario es responsable de su clave y de su consumo.
3. Jurisdicción: Florida, EE.UU. Ley aplicable y foro.
4. Nombre legal desde siteConfig.LEGAL_ENTITY, marcado visible si es PLACEHOLDER.
5. /cookies: sin analytics, dicho como feature de privacidad.
6. Fecha de actualización visible en las tres páginas, desde LEGAL_UPDATED en
   site.config.ts, no hardcodeada.
7. Bilingüe con paridad testeada. Metadata/OG por página e idioma. Sitemap.
8. Enlaces cruzados entre las tres páginas y desde el footer.
9. Los textos deben ser completos por sí mismos, no un esqueleto con "consulta
   a un abogado".

## Cierre
npm run build limpio, npm run test:i18n en verde, npm run lint limpio,
reporte de 5 líneas. No empieces la Fase 8.