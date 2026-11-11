//! Client-side voice activity detection for the transcription session.
//!
//! The transcription models do not detect turns on the server, so the client
//! decides where a turn ends: it streams audio while somebody is speaking and
//! tells the caller to commit once they pause. Nothing is sent while the call
//! is silent, which also keeps silence from being billed.
//!
//! Pure: no I/O and no clock (time is counted from the audio itself), so it is
//! unit-tested directly.

use crate::audio::AudioChunk;
use std::collections::VecDeque;

/// RMS level (of a 16-bit sample) above which a chunk counts as speech.
/// System audio is digital silence when nothing plays, so this only has to
/// clear codec noise, not a room's background.
const SPEECH_RMS: f64 = 250.0;
/// Audio kept from just before speech starts, so the first syllable is not cut.
const PREROLL_MS: u64 = 300;
/// A pause this long ends the turn.
const END_SILENCE_MS: u64 = 700;
/// A turn with less speech than this is noise (a click, a notification sound).
const MIN_SPEECH_MS: u64 = 200;
/// Somebody talking without pausing is still cut here, so text keeps arriving.
const MAX_TURN_MS: u64 = 20_000;

/// What to do with the audio buffered on the server after sending `send`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnEnd {
    /// The turn goes on (or nothing is being sent).
    None,
    /// A turn ended: commit the buffer to get its transcript.
    Commit,
    /// What was sent was only noise: clear the buffer instead.
    Discard,
}

#[derive(Debug, PartialEq)]
pub struct VadOutput {
    /// Chunks to append to the server's buffer, in order.
    pub send: Vec<AudioChunk>,
    pub end: TurnEnd,
}

#[derive(Debug, Default)]
pub struct Vad {
    speaking: bool,
    preroll: VecDeque<AudioChunk>,
    preroll_ms: u64,
    /// Since the turn started: all audio sent, and the loud part of it.
    turn_ms: u64,
    speech_ms: u64,
    /// The current run of quiet audio.
    silence_ms: u64,
}

fn duration_ms(chunk: &AudioChunk) -> u64 {
    let frames = chunk.pcm_s16le.len() as u64 / 2 / u64::from(chunk.channels.max(1));
    frames * 1000 / u64::from(chunk.sample_rate.max(1))
}

fn rms(chunk: &AudioChunk) -> f64 {
    let samples = chunk.pcm_s16le.as_chunks::<2>().0.iter().map(|b| f64::from(i16::from_le_bytes(*b)));
    let (sum, count) = samples.fold((0.0, 0u64), |(sum, count), s| (sum + s * s, count + 1));
    if count == 0 {
        0.0
    } else {
        (sum / count as f64).sqrt()
    }
}

impl Vad {
    pub fn push(&mut self, chunk: AudioChunk) -> VadOutput {
        let ms = duration_ms(&chunk);
        let loud = rms(&chunk) >= SPEECH_RMS;

        if !self.speaking {
            self.preroll_ms += ms;
            self.preroll.push_back(chunk);
            if !loud {
                // Keep only the most recent audio, always at least one chunk.
                while self.preroll.len() > 1 && self.preroll_ms > PREROLL_MS {
                    if let Some(old) = self.preroll.pop_front() {
                        self.preroll_ms -= duration_ms(&old);
                    }
                }
                return VadOutput { send: Vec::new(), end: TurnEnd::None };
            }
            self.speaking = true;
            self.turn_ms = self.preroll_ms;
            self.speech_ms = ms;
            self.silence_ms = 0;
            self.preroll_ms = 0;
            return VadOutput { send: self.preroll.drain(..).collect(), end: TurnEnd::None };
        }

        self.turn_ms += ms;
        if loud {
            self.speech_ms += ms;
            self.silence_ms = 0;
        } else {
            self.silence_ms += ms;
        }

        let end = if self.silence_ms >= END_SILENCE_MS {
            self.speaking = false;
            if self.speech_ms >= MIN_SPEECH_MS {
                TurnEnd::Commit
            } else {
                TurnEnd::Discard
            }
        } else if self.turn_ms >= MAX_TURN_MS {
            // Still speaking: cut here and carry on as a new turn.
            self.turn_ms = 0;
            self.speech_ms = 0;
            TurnEnd::Commit
        } else {
            TurnEnd::None
        };
        VadOutput { send: vec![chunk], end }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 100 ms of mono 24 kHz audio at a constant level.
    fn chunk(level: i16) -> AudioChunk {
        let pcm: Vec<u8> = std::iter::repeat_n(level.to_le_bytes(), 2400).flatten().collect();
        AudioChunk { pcm_s16le: pcm, sample_rate: 24_000, channels: 1 }
    }
    const LOUD: i16 = 4000;
    const QUIET: i16 = 0;

    #[test]
    fn silence_is_never_sent() {
        let mut vad = Vad::default();
        for _ in 0..50 {
            let out = vad.push(chunk(QUIET));
            assert!(out.send.is_empty());
            assert_eq!(out.end, TurnEnd::None);
        }
    }

    #[test]
    fn speech_is_sent_with_its_preroll_and_committed_after_a_pause() {
        let mut vad = Vad::default();
        for _ in 0..10 {
            vad.push(chunk(QUIET));
        }
        // Speech starts: the 300 ms before it go out with the first loud chunk.
        let first = vad.push(chunk(LOUD));
        assert_eq!(first.send.len(), 4);
        assert_eq!(first.end, TurnEnd::None);
        for _ in 0..5 {
            let out = vad.push(chunk(LOUD));
            assert_eq!((out.send.len(), out.end), (1, TurnEnd::None));
        }
        // 600 ms of pause is not the end yet; the 700th ms is.
        for _ in 0..6 {
            assert_eq!(vad.push(chunk(QUIET)).end, TurnEnd::None);
        }
        assert_eq!(vad.push(chunk(QUIET)).end, TurnEnd::Commit);
        // Back to idle: silence is not sent again.
        assert!(vad.push(chunk(QUIET)).send.is_empty());
    }

    #[test]
    fn a_short_pause_inside_a_sentence_does_not_end_the_turn() {
        let mut vad = Vad::default();
        vad.push(chunk(LOUD));
        for _ in 0..3 {
            for _ in 0..5 {
                assert_eq!(vad.push(chunk(QUIET)).end, TurnEnd::None);
            }
            assert_eq!(vad.push(chunk(LOUD)).end, TurnEnd::None);
        }
    }

    #[test]
    fn a_click_is_discarded_instead_of_transcribed() {
        let mut vad = Vad::default();
        vad.push(chunk(LOUD)); // 100 ms of sound, under the minimum
        let mut ends = Vec::new();
        for _ in 0..7 {
            ends.push(vad.push(chunk(QUIET)).end);
        }
        assert_eq!(ends.last(), Some(&TurnEnd::Discard));
        assert!(!ends.contains(&TurnEnd::Commit));
    }

    #[test]
    fn nonstop_speech_is_cut_at_the_maximum_turn_length() {
        let mut vad = Vad::default();
        let commits = (0..450).filter(|_| vad.push(chunk(LOUD)).end == TurnEnd::Commit).count();
        assert_eq!(commits, 2); // 45 s of speech: cut at 20 s and at 40 s
    }
}
