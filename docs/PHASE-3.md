# Fase 3 — /pricing

Lee docs/BRIEF-WEB.md para el contexto del producto.

## Requisitos
1. Mensaje principal: CloakView es GRATIS en v1. Sin suscripción, sin cuenta.
   El usuario solo paga su consumo de API a OpenAI con su propia key.
2. Estimador de costo mensual de API con tres perfiles: ligero, moderado,
   intensivo. Cada perfil muestra horas de reunión/mes asumidas y el rango
   de coste estimado.
3. Los precios de OpenAI deben estar marcados explícitamente como
   "aprox., según tarifas vigentes de OpenAI" y con fecha de referencia.
   No inventes cifras exactas: usa rangos.
4. Nota de roadmap visible: "Pro de pago único con features avanzadas —
   visión, traducción, modo offline — llegará en v1.1".
5. Nada de planes ficticios, nada de "Enterprise", nada de comparativas
   de tres columnas con la del medio destacada.
6. Todo el copy en en.json y es.json. Cero strings hardcodeados.
7. Reutiliza la paleta y los componentes existentes (features-grid, etc.).
   No dupliques estilos.

## Cie## Cie## Cie build` limpio, `npm run test:i18n` en verde, reporte de 5 líneas.
No empieces la Fase 4.

## Cifras de referencia (usar estas, no inventar otras)
- Transcripción gpt-4o-transcribe: ~$0.006/min de audio.
- Chat gpt-5-mini: ~$0.25/M tokens entrada, ~$2/M salida.
- Ligero: 10 h/mes ≈ $3-5. Moderado: 40 h/mes ≈ $14-20.
  Intensivo: 100 h/mes ≈ $36-50.
- Etiquetar todo como "aprox., tarifas de OpenAI a nov 2026".
