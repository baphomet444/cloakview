# macOS capture notes (screencapturekit 0.2.8)

> 2026-11-14: `src/capture_macos.rs` and the `sck` feature were REMOVED.
> The safe wrapper's `CMSampleBuffer::new` calls `get_frame_info()` on every
> buffer, which panics for audio buffers before any handler code runs, so the
> crate's capture would likely abort the app. The app keeps its proven
> `audio/screencapturekit.rs` source (built directly on `screencapturekit-sys`)
> and adapts its output to `cv_providers::audio::AudioChunk` at the call site.
> The notes below are kept for the pinned dependency's other quirks.

## What is verified and what is not
- `src/capture_macos.rs` (feature `sck`) was TYPE-CHECKED against the real
  screencapturekit 0.2.8 / -sys 0.2.8 sources on Linux (with temporary stubs for
  Apple-only linking). It has NEVER been run. Runtime behaviour (permissions,
  callbacks, real audio) must be tested on your Mac.
- `sck_audio.rs`, `audio.rs`, `vad.rs`, `pipeline.rs` are unit-tested.
- The microphone snippet below is NOT compiled anywhere.

## Quirks of 0.2.8 found by reading its source
1. `sample_rate`, `channel_count` and `excludes_current_process_audio` in
   `SCStreamConfiguration` are silently ignored (the sys layer only sets
   capturesAudio). You get SCK defaults and your own app's audio is NOT
   excluded. If CloakView never plays sound this is harmless; verify with a test.
   A newer version, or a vendored patch of -sys, is the fix if it matters.
2. `get_av_audio_buffer_list()` calls `panic!()` on error. Inside an Objective-C
   callback that would abort the app, so `capture_macos.rs` wraps it in
   `catch_unwind`.
3. `CopiedAudioBuffer.number_channels` is really the buffer count. We read the
   stream's format description (sample rate, channels, float, planar) instead.
4. By my reading, `copy_audio_buffers` copies a fixed 8-buffer struct out of a
   smaller allocation (a small heap over-read). Very unlikely to crash, but it
   is UB in the dependency. Another reason to plan an upgrade after launch.
5. `SCShareableContent::current()` unwraps; use `try_current()` (as we do) so a
   missing Screen Recording permission becomes a normal error you can show.
6. A display filter always captures video too. We use 100x100 at 1 fps and
   register only the audio output, so frames are dropped cheaply.

## Using it
```rust
// Cargo: cv_providers = { path = "...", features = ["sck"] }  (macOS only)
let (sys_tx, sys_rx) = tokio::sync::mpsc::channel(32);
let capture = cv_providers::capture_macos::SystemAudioCapture::start(sys_tx)?; // Err = no permission
pipeline.spawn_source(AudioSource::SystemAudio, sys_rx, out_tx.clone());
// later: capture.stop();
```
Keep `capture` in a `Mutex<Option<SystemAudioCapture>>` in Tauri state.

## Microphone (your side), UNVERIFIED sketch with cpal 0.15
```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
let dev = cpal::default_host().default_input_device().ok_or("no mic")?;
let cfg = dev.default_input_config().map_err(|e| e.to_string())?;
let (ch, rate) = (cfg.channels(), cfg.sample_rate().0);
let tx = mic_tx.clone();
let stream = dev.build_input_stream(
    &cfg.into(),
    move |data: &[f32], _| { let _ = tx.try_send(AudioChunk { samples: data.to_vec(), channels: ch, sample_rate: rate }); },
    |e| eprintln!("[cv] mic error: {e}"),
    None,
).map_err(|e| e.to_string())?;
stream.play().map_err(|e| e.to_string())?;
// keep `stream` alive (it is !Send: own it on a dedicated thread like SystemAudioCapture)
```
Check the sample format first (`cfg.sample_format()`): handle I16 if it is not F32.

## Packaging (notarized app)
- Screen Recording permission (TCC) for ScreenCaptureKit.
- `NSMicrophoneUsageDescription` in Info.plist.
- Hardened runtime entitlement `com.apple.security.device.audio-input`.

## Known gaps
- Energy VAD is crude in noisy rooms (swap in Silero/WebRTC VAD behind the same signature).
- Whisper-family models can invent text on near-silence: raise `rms_threshold` / `min_frames`.
