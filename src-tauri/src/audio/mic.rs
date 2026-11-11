use crate::audio::pcm::{downmix, to_s16le, Resampler};
use crate::audio::{AudioChunk, AudioSource, AudioStatus, EchoGate, TARGET_SAMPLE_RATE};
use crate::error::{AppError, AppResult};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat};
use std::sync::mpsc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;

/// The microphone, as an `AudioSource`: the user's own voice.
///
/// The cpal stream is not `Send`, so it lives on a thread of its own, which
/// `start` waits on just long enough to learn whether the stream opened.
pub struct MicAudioSource {
    gate: EchoGate,
    stop: Option<mpsc::Sender<()>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl MicAudioSource {
    pub fn new(gate: EchoGate) -> Self {
        Self { gate, stop: None, thread: None }
    }
}

/// Turns device buffers into ~100 ms chunks of 24 kHz mono 16-bit audio.
struct Sink {
    channels: usize,
    resampler: Resampler,
    pending: Vec<f32>,
    out: Sender<AudioStatus>,
    gate: EchoGate,
}

impl Sink {
    fn push<T: Sample>(&mut self, data: &[T])
    where
        f32: FromSample<T>,
    {
        // While the other side is heard on the speakers the microphone hears them
        // too; drop that audio instead of transcribing the call twice.
        if self.gate.mic_is_muted(crate::db::now_ms()) && !gate_disabled() {
            self.pending.clear();
            return;
        }
        let floats: Vec<f32> = data.iter().map(|&s| f32::from_sample(s)).collect();
        let mono = downmix(&floats, self.channels);
        self.pending.extend(self.resampler.process(&mono));
        let chunk_samples = (TARGET_SAMPLE_RATE / 10) as usize;
        while self.pending.len() >= chunk_samples {
            let block: Vec<f32> = self.pending.drain(..chunk_samples).collect();
            let chunk = AudioChunk { pcm_s16le: to_s16le(&block), sample_rate: TARGET_SAMPLE_RATE, channels: 1 };
            // Never block the audio thread: if the session is behind, this chunk is dropped.
            let _ = self.out.try_send(AudioStatus::Chunk(chunk));
        }
    }
}

/// Debug builds can switch the echo gate off, to test the microphone with the
/// speakers playing a recorded voice. Release builds always gate.
fn gate_disabled() -> bool {
    #[cfg(debug_assertions)]
    {
        std::env::var_os("CLOAKVIEW_MIC_NOGATE").is_some()
    }
    #[cfg(not(debug_assertions))]
    {
        false
    }
}

fn open(out: Sender<AudioStatus>, gate: EchoGate) -> Result<cpal::Stream, String> {
    let device = cpal::default_host().default_input_device().ok_or("no microphone found")?;
    let config = device.default_input_config().map_err(|e| e.to_string())?;
    let (channels, rate, format) = (config.channels() as usize, config.sample_rate(), config.sample_format());
    crate::diag::log(&format!("microphone: {rate} Hz, {channels} ch, {format}"));
    let mut sink = Sink { channels, resampler: Resampler::new(rate, TARGET_SAMPLE_RATE), pending: Vec::new(), out, gate };
    let on_error = |e: cpal::Error| crate::diag::log(&format!("microphone stream error: {e}"));
    let stream = match format {
        SampleFormat::F32 => {
            device.build_input_stream(config.into(), move |d: &[f32], _: &_| sink.push(d), on_error, None)
        }
        SampleFormat::I16 => {
            device.build_input_stream(config.into(), move |d: &[i16], _: &_| sink.push(d), on_error, None)
        }
        SampleFormat::I32 => {
            device.build_input_stream(config.into(), move |d: &[i32], _: &_| sink.push(d), on_error, None)
        }
        other => return Err(format!("unsupported microphone sample format {other}")),
    }
    .map_err(|e| e.to_string())?;
    stream.play().map_err(|e| e.to_string())?;
    Ok(stream)
}

impl AudioSource for MicAudioSource {
    fn start(&mut self, out: Sender<AudioStatus>) -> AppResult<()> {
        let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        let gate = self.gate.clone();
        let thread = std::thread::spawn(move || match open(out, gate) {
            Ok(stream) => {
                let _ = ready_tx.send(Ok(()));
                // Held until `stop` (or the source is dropped) closes the channel.
                let _ = stop_rx.recv();
                drop(stream);
            }
            Err(e) => {
                let _ = ready_tx.send(Err(e));
            }
        });
        match ready_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(Ok(())) => {
                self.stop = Some(stop_tx);
                self.thread = Some(thread);
                Ok(())
            }
            Ok(Err(e)) => Err(AppError::Audio(e)),
            Err(_) => Err(AppError::Audio("the microphone did not start".into())),
        }
    }

    fn stop(&mut self) -> AppResult<()> {
        drop(self.stop.take());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        Ok(())
    }
}
