# Sesión 2026-11-13: licencias de CloakView Pro y release 0.2.0

## Resultado
- **CloakView 0.2.0 publicado** (release `v0.2.0`, marcado como latest). DMG firmado, notarizado y grapado; Gatekeeper: `accepted source=Notarized Developer ID`.
  SHA-256 `f1205c1259ecf2279dc1751d75cb3991764fd1875f28048757235a2070737458` (igual para `CloakView_0.2.0_aarch64.dmg` y `CloakView-Pro_aarch64.dmg`; también en `checksums.txt`).
- `SHA256_ARM64` actualizado; `/en/download` y `/es/download` muestran el hash nuevo.

## Cambios
| Commit | Qué |
|---|---|
| `86cdc6b` | IDs reales de Lemon Squeezy (tienda 522202, producto 1524133). |
| `8a9e306` | La app rechaza claves de modo prueba (`license_key.test_mode`, comprobado contra la API real). Mensaje propio en en/es. |
| `7c765e3` | `PRO_CHECKOUT_URL` en `site.config.ts`: `null` = "Coming soon"; una URL = botón "Buy CloakView Pro". |
| `2efbe05` | Favicons de la web regenerados desde el logo recortado. `favicon.ico` seguía siendo la "N" de Next.js. |
| `d04be34` | Versión 0.2.0. |
| `349d320` | Hash de 0.2.0 en la web. |
| `3410c89` | Fuera el "cuando esté disponible" del texto de descarga y del changelog. |

## Verificado
- Revisión del merge: sin conflictos; `make_overlay` y `promote_overlay` intactos; History es panel y entra en `hideAll`.
- Atajo de leer pantalla (⌘⇧S) probado en el build 0.2.0, creando Ask y con Ask ya abierto: CloakView nunca pasa a ser la app activa.
- Gates: 125 tests Rust, 198 de la app, clippy limpio, 11 de la web, tsc y build.
- Compra de prueba en Lemon Squeezy ($99, modo test) y activate/validate/deactivate contra la API real.

## Hallazgo de seguridad
En **0.1.1**, cualquier texto introducido como clave desbloqueaba Pro (`PendingValidation` contaba como Pro y nunca se validaba). 0.2.0 lo corrige y los registros antiguos se cargan como inactivos. Las copias de 0.1.1 ya descargadas conservan el fallo (no hay actualizador automático). Recomendado: borrar los DMG de 0.1.1 de GitHub.

## Pendiente
- Lemon Squeezy: aprobación de la tienda → "Copy to Live Mode" → poner la URL de checkout **live** en `PRO_CHECKOUT_URL`.
- Decidir si se borran los DMG de 0.1.1.
- Decisiones abiertas: ⌘⇧H/⌘⇧L pisan Finder y Safari; push-to-ask no muestra el hub oculto; los fallos de atajo solo van al log; `promote_overlay` y push-to-ask fuera de design.md §5.
- `MARKETING.md` aún dice "gratis en v1"; con Pro a $99 el mensaje cambia.
- El buzón de soporte ya funciona (iCloud MX, SPF y DMARC activos): no hace falta ImprovMX.
