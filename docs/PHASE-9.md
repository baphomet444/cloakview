# Fase 9 — Redirección visual

Lee docs/BRIEF-WEB.md para el contexto del producto.

## Por qué
El sitio actual es plano y genérico: sin una sola capa de profundidad (solo
rellenos planos, bordes de 1px, rounded-lg), con la plantilla estándar de
badge + titular + subtítulo + dos botones, en max-w-5xl (ancho de blog) y con
la paleta azul-negro + índigo de cualquier SaaS de IA. Las seis secciones
comparten ancho y ritmo vertical, así que la página no respira distinto en
ningún punto.

El producto es un overlay invisible. El sitio debe sentirse como una capa
encima de algo, no como otra landing oscura.

## Dirección: cinemática cálida
- Carbón cálido de fondo, no azul-negro. Nada de #0b0e14.
- Un único acento cálido (ámbar/ember). Nunca dos acentos.
- Tipografía display de alto contraste para titulares, con leading cerrado
  (0.95-1.0) y pesos mezclados. El cuerpo sigue en la sans actual.
- Grano de película sutil sobre el fondo, a baja opacidad.
- Luz de borde cálida y sombras reales. Profundidad por elevación, no por
  gradientes de relleno.

## Composición
- Ancho: max-w-7xl. El hero es asimétrico a 12 columnas: copy en 5, mockup
  en las 7 restantes, sangrando por el borde derecho de la ventana y
  recortado por el viewport.
- El mockup deja de ser un diagrama: marco de ventana, sombra proyectada,
  rotación de 1-2 grados, luz de borde. Es el ancla visual de la página.
- Ritmo variable por sección: hero a sangre, cómo funciona en banda
  horizontal de 3 pasos numerados, features en bento (2 grandes + 4
  pequeñas, no una rejilla uniforme), costos en panel ancho único, FAQ
  estrecha en max-w-3xl.
- Fuera la píldora de badge. En su lugar, kicker en versalitas con un punto
  vivo.
- Botones cuadrados o de radio mínimo, con highlight interior superior.
  Nunca un relleno plano.

## Reglas
- Todo el copy sale de en.json/es.json con paridad exacta. Cero strings
  hardcodeados.
- No toques site.config.ts, el proxy, la lógica de descarga ni los tokens de
  color semánticos (background, surface, foreground, muted, accent, border):
  se redefinen sus valores, no sus nombres.
- Nada de imágenes de stock. El SVG se queda y se reestiliza.
- No introduzcas dependencias nuevas salvo una fuente de Google Fonts vía
  next/font.

## Verificación obligatoria
- npm run build limpio y npm run test:i18n en verde.
- Lighthouse en / y /es: las cuatro categorías en 100, sin bajar ninguna.
- Cero href="#".
- Comprobación visual: describe en el reporte cómo queda el hero a 1440px y
  a 390px.