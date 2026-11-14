use crate::{
    audio::{downmix_to_mono, f32_to_i16, AudioChunk, Resampler},
    vad::{Segmenter, VadConfig, SAMPLE_RATE},
    AudioSegment, AudioSource, Provider, Transcript,
};
use std::sync::Arc;
use tokio::{sync::mpsc, task::JoinHandle};

pub struct Pipeline {
    pub provider: Arc<dyn Provider>,
    pub language: Option<String>,
    pub vad: VadConfig,
}

impl Pipeline {
    /// Start listening on one source. Call once for the mic and once for system
    /// audio; both can share the same `out` sender. Returns the two task handles.
    ///
    /// Capture -> (mono, 16k, VAD) -> bounded queue -> sequential transcribe.
    /// The queue is bounded so a slow provider can never stall the capture
    /// callback; if it overflows we drop the segment rather than the audio thread.
    pub fn spawn_source(
        &self,
        source: AudioSource,
        mut rx: mpsc::Receiver<AudioChunk>,
        out: mpsc::Sender<Transcript>,
    ) -> (JoinHandle<()>, JoinHandle<()>) {
        let (seg_tx, mut seg_rx) = mpsc::channel::<Vec<i16>>(16);
        let vad = self.vad.clone();

        let segmenter_task = tokio::spawn(async move {
            let mut seg = Segmenter::new(vad);
            let mut resampler: Option<(u32, Resampler)> = None;
            while let Some(chunk) = rx.recv().await {
                let mono = downmix_to_mono(&chunk.samples, chunk.channels);
                let rs = match &mut resampler {
                    Some((rate, r)) if *rate == chunk.sample_rate => r,
                    _ => {
                        resampler = Some((chunk.sample_rate, Resampler::new(chunk.sample_rate, SAMPLE_RATE as u32)));
                        &mut resampler.as_mut().unwrap().1
                    }
                };
                let pcm = f32_to_i16(&rs.push(&mono));
                for s in seg.push(&pcm) {
                    if seg_tx.try_send(s).is_err() {
                        eprintln!("[cv] transcription queue full, dropping a segment");
                    }
                }
            }
        });

        let provider = self.provider.clone();
        let language = self.language.clone();
        let transcribe_task = tokio::spawn(async move {
            while let Some(pcm) = seg_rx.recv().await {
                let seg = AudioSegment { pcm16_mono_16k: pcm, source };
                match provider.transcribe(&seg, language.as_deref()).await {
                    Ok(t) if !t.text.is_empty() => {
                        if out.send(t).await.is_err() { break; }
                    }
                    Ok(_) => {}
                    Err(e) => eprintln!("[cv] transcribe error: {e}"),
                }
            }
        });

        (segmenter_task, transcribe_task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChatMessage, ProviderError, SttMode, SuggestionRequest};
    use async_trait::async_trait;

    struct Mock;
    #[async_trait]
    impl Provider for Mock {
        fn id(&self) -> &str { "mock" }
        fn supports_stt(&self) -> bool { true }
        fn stt_mode(&self) -> SttMode { SttMode::Batch }
        async fn transcribe(&self, seg: &AudioSegment, _l: Option<&str>) -> Result<Transcript, ProviderError> {
            Ok(Transcript { text: format!("{} samples", seg.pcm16_mono_16k.len()), source: seg.source })
        }
        async fn suggest(&self, _m: &SuggestionRequest) -> Result<String, ProviderError> { Ok(String::new()) }
    }

    fn chunk(ms: usize, loud: bool) -> AudioChunk {
        // 48 kHz stereo, like ScreenCaptureKit's default
        let n = 48_000 * ms / 1000;
        let mut v = Vec::with_capacity(n * 2);
        for i in 0..n {
            let s = if loud { (i as f32 * 0.05).sin() * 0.3 } else { 0.0 };
            v.push(s); v.push(s);
        }
        AudioChunk { samples: v, channels: 2, sample_rate: 48_000 }
    }

    #[tokio::test]
    async fn two_sources_are_tagged_and_transcribed() {
        let p = Pipeline { provider: Arc::new(Mock), language: None, vad: VadConfig::default() };
        let (out_tx, mut out_rx) = mpsc::channel(8);
        let (sys_tx, sys_rx) = mpsc::channel(8);
        let (mic_tx, mic_rx) = mpsc::channel(8);
        p.spawn_source(AudioSource::SystemAudio, sys_rx, out_tx.clone());
        p.spawn_source(AudioSource::Microphone, mic_rx, out_tx.clone());
        drop(out_tx);

        for (tx, _) in [(&sys_tx, 0), (&mic_tx, 1)] {
            for c in [chunk(300, false), chunk(1000, true), chunk(800, false)] {
                tx.send(c).await.unwrap();
            }
        }
        drop(sys_tx); drop(mic_tx);

        let mut got = vec![];
        while let Some(t) = out_rx.recv().await { got.push(t.source); }
        assert!(got.contains(&AudioSource::SystemAudio));
        assert!(got.contains(&AudioSource::Microphone));
        assert_eq!(got.len(), 2);
    }
}
