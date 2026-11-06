use crate::audio::{AudioSource, AudioStatus};
use crate::error::{AppError, AppResult};
use screencapturekit::sc_stream::SCStream;

/// macOS `AudioSource` backed by ScreenCaptureKit.
///
/// `new()` is the real, testable seam: it gates on the OS screen-recording
/// permission via `CGPreflightScreenCaptureAccess` before anything else runs.
/// `start()` is a deliberate stub for this task — wiring SCShareableContent /
/// SCContentFilter / SCStreamConfiguration / SCStreamOutput and converting
/// CMSampleBuffer audio into `AudioChunk`s is headless-CI-hostile and is left
/// for the manual-verification pass (Task 11 checklist), per the plan's
/// "single intentional seam" note for this task.
pub struct SckAudioSource {
    stream: Option<SCStream>,
}

impl SckAudioSource {
    pub fn new() -> AppResult<Self> {
        // Safe to call directly: objc2-core-graphics wraps the C-unwind FFI
        // call internally, so the exported fn itself is not `unsafe`.
        if !objc2_core_graphics::CGPreflightScreenCaptureAccess() {
            return Err(AppError::PermissionDenied);
        }
        Ok(Self { stream: None })
    }
}

impl AudioSource for SckAudioSource {
    fn start(&mut self, out: tokio::sync::mpsc::Sender<AudioStatus>) -> AppResult<()> {
        let _ = out;
        Err(AppError::Audio(
            "SckAudioSource::start is a stub pending the SCStream wiring adaptation (see task notes)"
                .into(),
        ))
    }

    fn stop(&mut self) -> AppResult<()> {
        // No stream is ever created by the current stub `start()`, so there
        // is nothing to tear down yet; `self.stream.take()` keeps the field
        // meaningful once `start()` is implemented.
        let _ = self.stream.take();
        Ok(())
    }
}
