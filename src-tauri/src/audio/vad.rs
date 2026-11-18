//! Client-side voice activity detection for the transcription session.
//!
//! The transcription models do not detect turns on the server, so the client
//! decides where a turn ends: it streams audio while somebody is speaking and
//! tells the caller to commit once they pause. Nothing is sent while the call
//! is silent, which also keeps silence from being billed.
//!
//! Pure: no I/O and no clock (time is counted from the audio itself), so it is
//! unit-tested directly.
//!
//! **Background noise.** A fixed loudness bar is not enough: when the other side
//! has a fan, traffic or a hum behind them, that noise clears the bar on its
//! own, the turn never ends, and every cut sends noise to be transcribed — which
//! the model turns into invented sentences, often in another language. So the
//! bar also follows the background: it is the quietest level heard over the last
//! few seconds (`FLOOR_WINDOW_MS`), times `NOISE_MARGIN`. Speech always dips
//! between words and breaths, so the quietest moment is the background, never the
//! voice; a steady noise becomes the floor and stops counting as speech.

use crate::audio::AudioChunk;
use std::collections::VecDeque;

/// RMS level (of a 16-bit sample) above which a chunk counts as speech, at the
/// least. System audio is digital silence when nothing plays, so this only has to
/// clear codec noise; a noisy background raises the bar (see the module doc).
pub(crate) const SPEECH_RMS: f64 = 250.0;
/// Audio kept from just before speech starts, so the first syllable is not cut.
const PREROLL_MS: u64 = 300;
/// A pause this long ends the turn.
const END_SILENCE_MS: u64 = 700;
/// A turn with less speech than this is noise (a click, a cough, a door).
const MIN_SPEECH_MS: u64 = 300;
/// Somebody talking without pausing is still cut here, so text keeps arriving.
const MAX_TURN_MS: u64 = 20_000;
/// How much audio the background estimate looks back over.
const FLOOR_WINDOW_MS: u64 = 3_000;
/// The estimate is used once it has seen this much audio; before that, only `SPEECH_RMS` applies.
const FLOOR_MIN_MS: u64 = 1_000;
/// How far above the background (its quietest level) speech has to be, as a
/// ratio of RMS levels: about 10 dB, which also covers the background's own swings.
const NOISE_MARGIN: f64 = 3.0;

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

#[derive(Debug)]
pub struct Vad {
    /// RMS level above which a chunk counts as speech.
    threshold: f64,
    speaking: bool,
    preroll: VecDeque<AudioChunk>,
    preroll_ms: u64,
    /// Since the turn started: all audio sent.
    turn_ms: u64,
    /// The level and length of every chunk sent in this turn, so its end can
    /// re-judge it against the background as known by then (`had_speech`).
    turn_levels: Vec<(f64, u64)>,
    /// The current run of quiet audio.
    silence_ms: u64,
    /// Level and length of the recent chunks, for the background estimate.
    recent: VecDeque<(f64, u64)>,
    recent_ms: u64,
}

fn duration_ms(chunk: &AudioChunk) -> u64 {
    let frames = chunk.pcm_s16le.len() as u64 / 2 / u64::from(chunk.channels.max(1));
    frames * 1000 / u64::from(chunk.sample_rate.max(1))
}

pub(crate) fn rms(chunk: &AudioChunk) -> f64 {
    let samples = chunk.pcm_s16le.as_chunks::<2>().0.iter().map(|b| f64::from(i16::from_le_bytes(*b)));
    let (sum, count) = samples.fold((0.0, 0u64), |(sum, count), s| (sum + s * s, count + 1));
    if count == 0 {
        0.0
    } else {
        (sum / count as f64).sqrt()
    }
}

impl Default for Vad {
    fn default() -> Self {
        Self::with_threshold(SPEECH_RMS)
    }
}

impl Vad {
    /// For a source noisier than digital system audio, such as a microphone.
    pub fn with_threshold(threshold: f64) -> Self {
        Self {
            threshold,
            speaking: false,
            preroll: VecDeque::new(),
            preroll_ms: 0,
            turn_ms: 0,
            turn_levels: Vec::new(),
            silence_ms: 0,
            recent: VecDeque::new(),
            recent_ms: 0,
        }
    }

    /// The level a chunk must reach to count as speech right now: the fixed
    /// threshold, or `NOISE_MARGIN` times the background when that is higher.
    fn speech_level(&self) -> f64 {
        if self.recent_ms < FLOOR_MIN_MS {
            return self.threshold;
        }
        let floor = self.recent.iter().map(|(level, _)| *level).fold(f64::INFINITY, f64::min);
        self.threshold.max(floor * NOISE_MARGIN)
    }

    /// Adds a chunk to the background estimate, forgetting what is older than the window.
    fn remember(&mut self, level: f64, ms: u64) {
        self.recent.push_back((level, ms));
        self.recent_ms += ms;
        while self.recent.len() > 1 && self.recent_ms - self.recent.front().map_or(0, |(_, ms)| *ms) >= FLOOR_WINDOW_MS {
            if let Some((_, old)) = self.recent.pop_front() {
                self.recent_ms -= old;
            }
        }
    }

    /// Whether the turn held enough speech, judged against the background as it
    /// is known now. A noise that starts mid-call counts as speech for the few
    /// seconds before it becomes the floor; by the end of that turn it is the
    /// floor, so the turn is discarded instead of sent to be transcribed.
    fn had_speech(&self) -> bool {
        let bar = self.speech_level();
        let speech_ms: u64 = self.turn_levels.iter().filter(|(level, _)| *level >= bar).map(|(_, ms)| ms).sum();
        speech_ms >= MIN_SPEECH_MS
    }

    pub fn push(&mut self, chunk: AudioChunk) -> VadOutput {
        let ms = duration_ms(&chunk);
        let level = rms(&chunk);
        self.remember(level, ms);
        let loud = level >= self.speech_level();

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
            self.turn_levels = self.preroll.iter().map(|c| (rms(c), duration_ms(c))).collect();
            self.silence_ms = 0;
            self.preroll_ms = 0;
            return VadOutput { send: self.preroll.drain(..).collect(), end: TurnEnd::None };
        }

        self.turn_ms += ms;
        self.turn_levels.push((level, ms));
        if loud {
            self.silence_ms = 0;
        } else {
            self.silence_ms += ms;
        }

        let end = if self.silence_ms >= END_SILENCE_MS {
            self.speaking = false;
            if self.had_speech() {
                TurnEnd::Commit
            } else {
                TurnEnd::Discard
            }
        } else if self.turn_ms >= MAX_TURN_MS {
            // Still going: cut here and carry on as a new turn.
            let end = if self.had_speech() { TurnEnd::Commit } else { TurnEnd::Discard };
            self.turn_ms = 0;
            self.turn_levels.clear();
            end
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

    /// Speech as it really sounds: loud, with a short dip between words.
    fn speech(i: usize, level: i16, dip: i16) -> AudioChunk {
        chunk(if i % 4 == 3 { dip } else { level })
    }

    #[test]
    fn nonstop_speech_is_cut_at_the_maximum_turn_length() {
        let mut vad = Vad::default();
        let commits = (0..450).filter(|&i| vad.push(speech(i, LOUD, QUIET)).end == TurnEnd::Commit).count();
        assert_eq!(commits, 2); // 45 s of speech: cut at 20 s and at 40 s
    }

    #[test]
    fn a_steady_background_noise_is_not_speech() {
        // A fan or a hum on the other side: louder than the fixed threshold, but steady.
        let mut vad = Vad::default();
        let outputs: Vec<VadOutput> = (0..100).map(|_| vad.push(chunk(1000))).collect();
        assert!(outputs.iter().all(|o| o.end != TurnEnd::Commit), "noise was sent to be transcribed");
        // Once it is known as the background, nothing is sent at all.
        assert!(outputs[20..].iter().all(|o| o.send.is_empty() && o.end == TurnEnd::None));
    }

    #[test]
    fn a_noise_that_starts_mid_call_is_discarded_once_it_is_known() {
        let mut vad = Vad::default();
        for _ in 0..30 {
            vad.push(chunk(QUIET));
        }
        let ends: Vec<TurnEnd> = (0..100).map(|_| vad.push(chunk(1000)).end).collect();
        assert!(!ends.contains(&TurnEnd::Commit));
        assert!(ends.contains(&TurnEnd::Discard));
    }

    #[test]
    fn speech_over_a_steady_background_is_still_heard() {
        let mut vad = Vad::default();
        let mut ends = Vec::new();
        ends.extend((0..50).map(|_| vad.push(chunk(600)).end));
        ends.extend((0..20).map(|i| vad.push(speech(i, 6000, 600)).end));
        ends.extend((0..10).map(|_| vad.push(chunk(600)).end));
        assert_eq!(ends.iter().filter(|e| **e == TurnEnd::Commit).count(), 1);
        assert_eq!(ends.last(), Some(&TurnEnd::None));
    }
}
