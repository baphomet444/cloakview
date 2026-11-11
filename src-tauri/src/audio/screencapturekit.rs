use crate::audio::{AudioChunk, AudioSource, AudioStatus, TARGET_SAMPLE_RATE};
use crate::error::{AppError, AppResult};
use screencapturekit::sc_stream_configuration::SCStreamConfiguration;
use screencapturekit_sys::cm_sample_buffer_ref::CMSampleBufferRef;
use screencapturekit_sys::content_filter::{UnsafeContentFilter, UnsafeInitParams};
use screencapturekit_sys::os_types::rc::Id;
use screencapturekit_sys::shareable_content::UnsafeSCShareableContent;
use screencapturekit_sys::stream::UnsafeSCStream;
use screencapturekit_sys::stream_error_handler::UnsafeSCStreamError;
use screencapturekit_sys::stream_output_handler::UnsafeSCStreamOutput;
use tokio::sync::mpsc::Sender;

/// macOS `AudioSource` backed by ScreenCaptureKit: the system audio, which is
/// what the other participants of a call sound like on this Mac.
///
/// ScreenCaptureKit only hands out audio as part of a screen stream, so this
/// opens one on the main display at the smallest size it accepts and ignores
/// its frames. That is also why the permission involved is screen recording.
///
/// The stream is driven through `screencapturekit-sys`, not the `screencapturekit`
/// wrapper: the wrapper (0.2.8) reads video frame information from every sample
/// buffer, which an audio buffer does not have, and panics on the first one.
pub struct SckAudioSource {
    stream: Option<Id<UnsafeSCStream>>,
}

/// `SCStreamOutputType.audio`.
const OUTPUT_TYPE_AUDIO: u8 = 1;

impl SckAudioSource {
    /// Fails with `PermissionDenied` unless screen recording is allowed.
    ///
    /// When it is not, this also asks macOS for it: the first time that shows
    /// the system prompt, and every time it lists the app under Screen & System
    /// Audio Recording in System Settings so the user can switch it on. macOS
    /// only applies the change after the app is reopened.
    pub fn new() -> AppResult<Self> {
        // Safe to call directly: objc2-core-graphics wraps the C-unwind FFI
        // calls internally, so the exported fns themselves are not `unsafe`.
        if !objc2_core_graphics::CGPreflightScreenCaptureAccess() {
            objc2_core_graphics::CGRequestScreenCaptureAccess();
            return Err(AppError::PermissionDenied);
        }
        Ok(Self { stream: None })
    }
}

/// Receives the stream's sample buffers on ScreenCaptureKit's own thread.
struct AudioOutput {
    out: Sender<AudioStatus>,
    /// Buffers received so far, for the diagnostics log.
    received: std::sync::atomic::AtomicU64,
}

impl UnsafeSCStreamOutput for AudioOutput {
    fn did_output_sample_buffer(&self, sample: Id<CMSampleBufferRef>, of_type: u8) {
        if of_type != OUTPUT_TYPE_AUDIO {
            return;
        }
        // Mono is requested, so the first buffer is the whole signal.
        let buffers = sample.get_av_audio_buffer_list();
        let Some(buffer) = buffers.first() else { return };
        let pcm = f32le_to_s16le(&buffer.data);
        if pcm.is_empty() {
            return;
        }
        // Proof that audio is arriving at all, and then roughly once a minute.
        let n = self.received.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if n == 0 || n.is_multiple_of(3000) {
            crate::diag::log(&format!("audio buffer #{n} ({} bytes)", pcm.len()));
        }
        let chunk = AudioChunk { pcm_s16le: pcm, sample_rate: TARGET_SAMPLE_RATE, channels: 1 };
        // Never block the capture thread: if the session is behind, this chunk is dropped.
        let _ = self.out.try_send(AudioStatus::Chunk(chunk));
    }
}

struct ErrorOutput {
    out: Sender<AudioStatus>,
}

impl UnsafeSCStreamError for ErrorOutput {
    fn handle_error(&self) {
        let _ = self.out.try_send(AudioStatus::Error("the system audio capture stopped".into()));
    }
}

/// ScreenCaptureKit delivers 32-bit float samples; the transcription session takes 16-bit PCM.
fn f32le_to_s16le(bytes: &[u8]) -> Vec<u8> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|b| {
            let sample = f32::from_le_bytes(*b);
            let scaled = (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16;
            scaled.to_le_bytes()
        })
        .collect()
}

impl AudioSource for SckAudioSource {
    fn start(&mut self, out: Sender<AudioStatus>) -> AppResult<()> {
        let content = UnsafeSCShareableContent::get().map_err(AppError::Audio)?;
        let display = content
            .displays()
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Audio("no display to capture audio from".into()))?;

        let config = SCStreamConfiguration {
            // The video side is required but unused.
            width: 2,
            height: 2,
            captures_audio: true,
            sample_rate: TARGET_SAMPLE_RATE,
            channel_count: 1,
            // The app's own sounds are not part of the conversation.
            excludes_current_process_audio: true,
            ..Default::default()
        };
        let filter = UnsafeContentFilter::init(UnsafeInitParams::Display(display));
        let stream = UnsafeSCStream::init(filter, config.into(), ErrorOutput { out: out.clone() });
        stream.add_stream_output(AudioOutput { out, received: Default::default() }, OUTPUT_TYPE_AUDIO);
        stream.start_capture().map_err(AppError::Audio)?;
        self.stream = Some(stream);
        Ok(())
    }

    fn stop(&mut self) -> AppResult<()> {
        match self.stream.take() {
            Some(stream) => stream.stop_capture().map_err(AppError::Audio),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float_samples_become_16_bit_and_are_clamped() {
        let floats: [f32; 5] = [0.0, 0.5, -0.5, 2.0, -2.0];
        let bytes: Vec<u8> = floats.iter().flat_map(|f| f.to_le_bytes()).collect();
        let out: Vec<i16> =
            f32le_to_s16le(&bytes).as_chunks::<2>().0.iter().map(|b| i16::from_le_bytes(*b)).collect();
        assert_eq!(out, vec![0, 16384, -16384, i16::MAX, -i16::MAX]);
    }

    #[test]
    fn a_trailing_partial_sample_is_ignored() {
        assert!(f32le_to_s16le(&[1, 2, 3]).is_empty());
    }
}
