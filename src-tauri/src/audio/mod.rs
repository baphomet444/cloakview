pub mod screencapturekit;
pub mod vad;

use crate::error::AppResult;

/// What the capture produces and the transcription session is told to expect (mono).
pub const TARGET_SAMPLE_RATE: u32 = 24_000;

/// Raw PCM audio chunk produced by an `AudioSource`.
///
/// `pcm_s16le` holds the little-endian byte encoding of 16-bit PCM samples
/// (2 bytes per sample), not individual sample values — hence `Vec<u8>`
/// rather than `Vec<i16>`.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioChunk {
    pub pcm_s16le: Vec<u8>,
    pub sample_rate: u32,
    pub channels: u32,
}

/// Item sent over the channel an `AudioSource` is given to drive.
#[derive(Debug)]
pub enum AudioStatus {
    Chunk(AudioChunk),
    Ended,
    Error(String),
}

/// A source of audio chunks. `start` must not block; it spawns its own task
/// and returns immediately. `stop` requests shutdown (best-effort).
pub trait AudioSource: Send {
    fn start(&mut self, out: tokio::sync::mpsc::Sender<AudioStatus>) -> AppResult<()>;
    fn stop(&mut self) -> AppResult<()>;
}

/// Test double: replays a fixed list of chunks, optionally injecting an
/// `AudioStatus::Error` after a given chunk index instead of continuing.
#[cfg(test)]
pub struct FakeAudioSource {
    pub chunks: Vec<AudioChunk>,
    pub fail_after: Option<usize>,
}

#[cfg(test)]
impl AudioSource for FakeAudioSource {
    fn start(&mut self, out: tokio::sync::mpsc::Sender<AudioStatus>) -> AppResult<()> {
        let chunks = std::mem::take(&mut self.chunks);
        let fail_after = self.fail_after;
        tokio::spawn(async move {
            for (i, c) in chunks.into_iter().enumerate() {
                if fail_after == Some(i) {
                    let _ = out.send(AudioStatus::Error("injected".into())).await;
                    return;
                }
                if out.send(AudioStatus::Chunk(c)).await.is_err() {
                    return;
                }
            }
            let _ = out.send(AudioStatus::Ended).await;
        });
        Ok(())
    }

    fn stop(&mut self) -> AppResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fake_source_emits_chunks_then_ended() {
        let mut src = FakeAudioSource {
            chunks: vec![AudioChunk { pcm_s16le: vec![0; 640], sample_rate: 16_000, channels: 1 }],
            fail_after: None,
        };
        let (tx, mut rx) = tokio::sync::mpsc::channel(8);
        src.start(tx).unwrap();
        match rx.recv().await {
            Some(AudioStatus::Chunk(_)) => {}
            other => panic!("expected chunk, got {other:?}"),
        }
        assert!(matches!(rx.recv().await, Some(AudioStatus::Ended)));
    }

    #[tokio::test]
    async fn fake_source_error_injection() {
        let mut src = FakeAudioSource {
            chunks: vec![
                AudioChunk { pcm_s16le: vec![], sample_rate: 16_000, channels: 1 };
                3
            ],
            fail_after: Some(1),
        };
        let (tx, mut rx) = tokio::sync::mpsc::channel(8);
        src.start(tx).unwrap();
        assert!(matches!(rx.recv().await, Some(AudioStatus::Chunk(_))));
        assert!(matches!(rx.recv().await, Some(AudioStatus::Error(_))));
    }
}
