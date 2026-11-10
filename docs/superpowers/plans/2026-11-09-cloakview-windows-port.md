# CloakView — Port a Windows

Estado: borrador. Se ejecuta despues de publicar y validar la version macOS.

## Objetivo

Build x86_64 e instalable NSIS/MSI firmado, con paridad funcional: captura de audio del sistema mas microfono, overlay excluido de la captura de pantalla, atajos globales y las mismas ventanas.

## No objetivos

- Paridad visual exacta. El overlay sigue las convenciones de Windows.
- ARM64 de Windows en la primera entrega.
- Store de Microsoft. La primera distribucion es descarga directa.

## Prerrequisitos

- macOS publicado y con al menos dos semanas de uso real.
- Cuenta de Azure con Trusted Signing aprobada (validacion de identidad de empresa, de una a tres semanas).
- Una maquina Windows 11 fisica para pruebas manuales. El CI no sustituye la prueba del overlay.

## Fase W1 — Backend de audio WASAPI

Archivo nuevo: `src-tauri/src/audio/wasapi.rs`, detras del trait `AudioSource` de la Task 8.

- Captura loopback del dispositivo de render por defecto con `IAudioClient3` y `AUDCLNT_STREAMFLAGS_LOOPBACK`.
- Captura simultanea de microfono por `IAudioCaptureClient`.
- Mezcla y remuestreo a 16 kHz mono PCM, con el mismo contrato de chunks que macOS.
- Reaccion a `IMMNotificationClient` ante cambios de dispositivo: reabrir sin cortar la sesion.
- Deteccion de silencio identica al backend macOS.

Aceptacion: `cargo test` con el backend simulado y prueba manual con musica y voz a la vez; el nivel mezclado debe moverse en ambos casos.

## Fase W2 — Exclusion de captura

Archivo nuevo: `src-tauri/src/windows_ext.rs`Archivo nuevo: `src-tauri/src/windows_ext.rs`Archivo nuxclude_from_capture(window: &tauri::WebviewWindow) -> AppResult<()> {
    let hwnd = window.hwnd().map_err(|e| AppError::Audio(e.to_string()))?;
    let ok = unsafe { SetWindowDisplayAffinity(hwnd.0, WDA_EXCLUDEFROMCAPTURE) };
    if ok == 0 { return Err(AppError::Audio("SetWindowDisplayAffinity".into())); }
    Ok(())
}