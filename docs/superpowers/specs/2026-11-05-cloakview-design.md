# CloakView — Copiloto de reuniones en tiempo real (BYOK-first)

**Fecha:** 2026-11-05
**Estado:** propuesto para aprobación
**Nombre:** CloakView (dominios cloakview.app, .ai, .io y .dev libres; cloakview.com está en venta — opcional comprarlo después)

---

## 1. Visión e intención

Aplicación de escritorio que escucha el audio de las reuniones del usuario
(Zoom, Meet, Teams, etc.), lo transcribe en tiempo real con IA, detecta cuándo
le toca hablar al usuario y le sugiere respuestas — todo con la **API key
propia del usuario** (BYOK), sin ningún backend nuestro.

**Principio rector:** *local-first, BYOK-first*. Todos los datos viven en el
Mac del usuario; la única red externa es el proveedor de IA que el usuario
configure. Esto elimina costos de servidor, infra de billing y la mayor parte
de la superficie de privacidad — y es la mejora estructural respecto a los
productos existentes del género, que cobran suscripción actuando de proxy.

## 2. Lo que NO es

- No es una copia de ningún producto existente: se escribe todo el código de
  cero (línea roja legal: ningún código, asset o marca de terceros).
- No tiene backend, cuentas, ni suscripciones en v1.
- No se distribuye en App Store (distribución directa firmada; las sandbox
  policies de la App Store romperían la captura de audio de otros procesos).
- v1 no incluye: visión de pantalla, traducción en tiempo real, Windows.

## 3. Usuario y criterios de éxito

Usuario: persona que participa en videollamadas frecuentes (entrevistas,
ventas, reuniones) y quiere asistencia en tiempo real usando su propia clave
de OpenAI.

Éxito del MVP (verificable):
1. App firmada y notarizada que arranca en macOS 14.2+.
2. Captura audio del sistema (y micrófono) tras conceder permiso.
3. Transcripción en tiempo real visible en el overlay (< 2 s de latencia).
4. Detección de turno/pregunta y generación de sugerencias con streaming.
5. Notas manuales y auto-notas persistidas entre sesiones.
6. La clave de API vive en Keychain, nunca en disco en claro ni en logs.

## 4. Stack

| Capa | Elección | Por qué |
|---|---|---|
| Shell | **Tauri 2.x** (Rust) | Binario pequeño, memoria baja, firmado+notarizado sin fricción |
| UI | **React 18 + Vite + TypeScript** | Ecosistema maduro para UI de overlay |
| Audio (macOS) | **ScreenCaptureKit** (crate `screencapturekit`) | API pública, captura audio del sistema + mic; sin APIs privadas |
| Audio (Windows, v2) | WASAPI loopback (crate `cpal`) | Equivalente funcional |
| IA en vivo | **OpenAI Realtime API** (WebSocket), modelo por defecto `gpt-4o-mini-realtime` | Transcripción + VAD server-side |
| IA texto | OpenAI **Responses API** con streaming | Sugerencias; modelo por defecto `gpt-4o-mini` |
| Transcripción offline (v2) | `whisper.cpp` (crate `whisper-rs`) | Modo $0/sin red |
| Datos | **SQLite** (`rusqlite`, WAL) | Notas, transcripciones, conversaciones |
| Secretos | **Keychain** (`keyring` crate) | La API key |
| Settings | JSON en `~/Library/Application Support/com.baphomet.cloakview/` | `tauri-plugin-store` |
| Atajos globales | `tauri-plugin-global-shortcut` | Quick-hide, push-to-ask |
| Licencia | MIT (decisión registrada; reversible antes del primer release público) | Facilita contribuciones si se open-sourcea |

## 5. Arquitectura

```
┌──────────────────────────────────────────────────────────┐
│ cloakview.app (Tauri 2)                                      │
│                                                          │
│  ┌─────────────┐  IPC (invoke/events)  ┌───────────────┐ │
│  │  Rust core  │ ◄──────────────────► │  React UI     │ │
│  │             │                      │  (overlay)    │ │
│  │ • audio     │                      └───────────────┘ │
│  │ • realtime  │                                          │
│  │ • answers   │   ┌──────────────┐                       │
│  │ • sqlite    │   │ Ventanas:    │  transparente,        │
│  │ • keychain  │   │ hub, listen, │  always-on-top,       │
│  │ • hotkeys   │   │ ask, notes   │  excluidas de captura │
│  └──────┬──────┘   └──────────────┘                       │
│         │                                                │
│         └──► wss://api.openai.com (clave del usuario)    │
└──────────────────────────────────────────────────────────┘
```

### Componentes Rust (crate `cloakview-core` dentro del repo Tauri)

1. **`audio`** — Abstracción `AudioSource`: implementación macOS con
   ScreenCaptureKit (SCStream con `capturesAudio = true`, mezcla sistema+mic,
   16 kHz mono PCM). Emite chunks por channel. Permission handling con guía
   en UI si `CGPreflightScreenCaptureAccess()` es falso.
2. **`realtime`** — Cliente del WebSocket Realtime: `session.update`
   (modelo, VAD turn detection, idiomas), audio in (base64 PCM), eventos out
   (`conversation.item.input_audio_transcription.completed`,
   `response.*`). Reconexión con backoff; sesión efímera no aplica (es clave
   del usuario, no ephemeral token).
3. **`answers`** — Responses API con streaming SSE: recibe transcripción
   reciente + contexto (CV/notas vinculadas), emite delta de sugerencia.
   Prompts versionados en código (system prompt editable en settings, v2).
4. **`detect`** — Heurística de turno: VAD server-side + keywords
   configurables ("¿tú qué opinas?", nombre del usuario) para resaltar cuándo
   le toca hablar.
5. **`store`** — SQLite en Application Support: tablas `notes`,
   `conversations`, `conversation_messages` (esquema propio, diseñado en la
   fase de plan).
6. **`secrets`** — `keyring` con service `cloakview`, account `openai-api-key`.
   Comandos: `save_api_key`, `has_api_key`, `clear_api_key`. Nunca loguear
   el valor.
7. **`windows`** — Manager de ventanas overlay: config Tauri
   (transparent, alwaysOnTop, decorations off, skipTaskbar) + en macOS
   `NSWindow.sharingType = .none` (vía `objc2-app-kit`) para exclusión de
   captura.
8. **`hotkeys`** — Registro global: Quick Hide, Push-to-Ask, Start/Stop
   Listen, toggle visibilidad (configurables en Settings).

### Comandos IPC (superficie frontend → Rust)

`get_api_key_status`, `save_api_key`, `clear_api_key`,
`start_listen`, `stop_listen`, `get_mic_status`,
`request_answer(question?)`, `stop_answer`,
CRUD notas (`create_note`, `list_notes`, `update_note`, `delete_note`,
folders/tags), CRUD conversaciones, `get_settings`/`save_settings`,
`get_app_info` (versión), `open_external(url)`.

Eventos Rust → frontend: `transcript-delta`, `turn-detected`,
`answer-delta`, `answer-done`, `listen-state`, `error`.

### Frontend (React)

- **Hub** — ventana flotante compacta (igual de translúcida que el resto):
  botones Listen / Ask / Notes / Opacidad / Settings, arrastrable.
- **ListenWindow** — transcripción en vivo con scroll, resaltado del turno
  detectado.
- **AskWindow** — input + sugerencias con streaming (Cmd+Shift+I push-to-ask).
- **NotesWindow** — CRUD de notas + auto-notas generadas al final de sesión.
- **Settings** — clave API (máscara, test de conexión), modelo, idiomas,
  atajos, opacidad, tema claro/oscuro, auto-borrado (default 30 días).
- Tema con `data-theme`, RTL no aplica v1. Todo texto en i18n-ready (es/en).

## 6. Flujo de una sesión Listen

1. Usuario pulsa Start Listen (atajo o hub). Rust pide permiso de captura de
   pantalla la primera vez; si ya concedido, arranca SCStream.
2. Audio (16 kHz mono) → chunks por channel → Realtime API.
3. Eventos de transcripción → frontend (`transcript-delta`); `detect` marca
   turnos → `turn-detected` resalta la UI.
4. Si el usuario pregunta (push-to-ask o click): `answers` envía los últimos
   30 turnos de transcripción + contexto seleccionado → streaming de
   sugerencia en AskWindow.
5. Stop Listen → cierre de WS, resumen opcional (auto-nota) → SQLite.

## 7. Privacidad y seguridad

- La clave API solo en Keychain; en memoria solo cuando se usa; rotación
  desde Settings. Nunca en logs, crash reports ni telemetría.
- **Telemetría: ninguna en v1.** Sin analytics, sin session replay. (Decisión
  explícita; si algún día se añade, será opt-in y documentado.)
- Transcripciones y notas: solo en el SQLite local del usuario. Auto-borrado
  configurable (default 30 días).
- Declaración de privacidad publicada en el repo (audio va al proveedor de IA
  que el usuario configure; nada más sale del dispositivo).

## 8. Distribución

- macOS: DMG firmado con Developer ID propio del usuario (Apple Developer
  $99/año) + `notarytool` + staple. Checksums SHA-256 publicados en cada
  release. Versión mínima macOS 14.2.
- Windows (v2): installer firmado con certificado Authenticode (OV).
- `Info.plist`: `NSScreenCaptureUsageDescription` y
  `NSMicrophoneUsageDescription` redactados.
- Versionado semver desde 0.1.0.

## 9. Fases

- **Fase MVP (este proyecto):** hub + overlay, captura audio, Realtime
  transcripción, turno, Ask con streaming, notas + SQLite, Keychain,
  settings, atajos, firma+notarización, CI de build.
- **v1.1:** auto-notas con resumen al cerrar sesión, export Markdown/PDF.
- **v2 (futuro, fuera de este spec):** visión (frames → vision API),
  traducción realtime, whisper.cpp offline, Windows, i18n completo.

## 10. Riesgos y mitigaciones

| Riesgo | Mitigación |
|---|---|
| ScreenCaptureKit requiere permiso de grabación de pantalla (UX de miedo) | Guía de onboarding + deep link a System Settings; fallback a solo-mic |
| Latencia Realtime > 2 s con mala red | Indicador de estado; modo "solo transcripción" local (v2 whisper) |
| Costos impredecibles en la clave del usuario | Medidor de uso estimado en Settings (tokens contados localmente) |
| Cambios de API de OpenAI | Capa `realtime`/`answers` aislada con traits; modelo configurable |
| Gatekeeper/quarantine en distribución directa | Notarización obligatoria en CI; test de `spctl -a -vv` en CI |
| macOS 15+ endurece ScreenCaptureKit | Compat CI en última beta de macOS antes de cada release |

## 11. Línea roja legal (registrada)

Ningún archivo, código, texto, asset o dato del análisis de Smog AI se copia,
importa o transforma en este proyecto. El análisis sirvió para identificar
técnicas (públicas) y mejoras de diseño; la implementación es clean-room
escrita desde cero, verificable en cada commit.
