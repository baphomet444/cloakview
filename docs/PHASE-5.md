# Fase 5 — /guide (documentación MDX)

Lee docs/BRIEF-WEB.md para el contexto del producto.

## Contexto
- Next 16.4.5 con App Router y [locale]. params es Promise.
- Paleta y componentes existentes. No dupliques estilos.
- MDX aún no está instalado. Justifica el paquete que elijas.

## Requisitos
1. Ruta /guide bilingüe (/en/guide + /es/guide) y subrutas por artículo.
   Contenido en MDX, un archivo por idioma por artículo.
2. Artículos mínimos:
   - Instalación (descarga, arrastrar a Aplicaciones, primer arranque)
   - Permisos de macOS: captura de pantalla y micrófono, con qué pasa si
     se deniegan y cómo revertirlo en Ajustes del Sistema
   - Crear y conectar la API key de OpenAI: paso a paso, incluido dónde
     se guarda localmente y cómo revocarla
   - Atajos globales
   - Troubleshooting: al menos 6 problemas reales con su solución
3. Índice lateral sticky con scroll-spy (marca la sección activa), bilingüe.
   Debe funcionar con teclado y anunciarse con aria-current.
4. Cada art4. Cada art4. Cada art4. Cada art4.nte, breadcrumb, y metadata/OG
   por idioma.
5. Los enlaces externos (openai.com, etc.) llevan rel="noopener noreferrer"
   y target="_blank" solo si es imprescindible. Prefiere abrir en la misma
   pestaña.
6. Cero strings hardcodeados en componentes: títulos, labels del índice y
   navegación van en en.json/es.json. El cuerpo del artículo va en MDX.
7. Los bloques de código en MDX con resaltado de sintaxis y botón de copiar
   accesible.
8. Sitemap actualizado con todas las rutas de guía en ambos idiomas.

## Cierre
`npm run build` limpio, `npm run test:i18n` en verde, reporte de 5 líneas.
No empieces la Fase 6.
