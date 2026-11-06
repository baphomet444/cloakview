# Fase 2 — LANDING (/)

Lee también docs/BRIEF-WEB.md para el contexto del producto.

## Contexto ya hecho en Fase 1
- Next 16.4.5, React 19.3, Tailwind 4 (configurado en CSS con @theme).
- Middleware en src/proxy.ts. params es Promise: `const { locale } = await params`.
- site.config.ts centraliza URLs. Usa isGithubPending() para el CTA de descarga.
- Diccionarios en src/dictionaries/en.json y es.json, con paridad testeada.
  `npm run test:i18n` debe seguir pasando al cerrar.

## Requisitos
1. Hero con mockup SVG ORIGINAL del overlay (ilustración propia en
   src/components/, no screenshots, no imágenes de stock). Debe representar
   el overlay sobre una videollamada: transcripción en vivo y una sugerencia
   de respuesta.
2. Sección "cómo funciona" en 3 pasos: conecta tu key → escucha → sugiere.
3. Grid de features: BYOK, local-first, overlay privado, atajos globales, es/en.
4. Sección de costos honesta: "pagas tu consumo de API, nosotros nada".
5. FAQ con 8 preguntas de un escéptico real (¿me estáis escuchando?,
   ¿qué pasa si revoco la key?, ¿funciona con Teams?, ¿dónde van las
   transcripciones?, etc.).
6. CTA de descarga que respete isGithubPending(): si el repo no existe,
   botón deshabilitado con texto "próximamente", NUNCA enlace roto.
7. TODO el copy en en.json y es.json. Cero strings hardcodeados en componentes.
8. Componentes en src/components/, no todo en page.tsx.
9. Accesibilidad: un solo h1, jerarquía de headings correcta, alt en el SVG
   o aria-hidden si es decorativo, contraste AA.

## Cierre
`npm run build` limpio, `npm run test:i18n` en verde, reporte de 5 líneas que
incluya la paleta elegida y por qué. No empieces la Fase 3.
