//! Energy-based voice activity detector that cuts a 16 kHz mono stream into
//! speech segments. Simple on purpose: swap for Silero/WebRTC VAD later
//! without touching anything else.

pub const SAMPLE_RATE: usize = 16_000;
const FRAME: usize = SAMPLE_RATE / 50; // 20 ms

#[derive(Clone, Debug)]
pub struct VadConfig {
    pub rms_threshold: f32,     // 0..1 of full scale
    pub start_frames: usize,    // consecutive loud frames to start a segment
    pub hangover_frames: usize, // quiet frames that end a segment
    pub min_frames: usize,      // discard shorter segments
    pub max_frames: usize,      // force a cut (keeps latency bounded)
}

impl Default for VadConfig {
    fn default() -> Self {
        Self { rms_threshold: 0.01, start_frames: 3, hangover_frames: 25, min_frames: 15, max_frames: 250 }
    }
}

pub struct Segmenter {
    cfg: VadConfig,
    carry: Vec<i16>,
    seg: Vec<i16>,
    speaking: bool,
    loud_run: usize,
    quiet_run: usize,
    pre_roll: Vec<i16>, // frames before the start trigger so we don't clip the first word
}

impl Segmenter {
    pub fn new(cfg: VadConfig) -> Self {
        Self { cfg, carry: vec![], seg: vec![], speaking: false, loud_run: 0, quiet_run: 0, pre_roll: vec![] }
    }

    fn rms(frame: &[i16]) -> f32 {
        let sum: f64 = frame.iter().map(|&s| (s as f64 / 32768.0).powi(2)).sum();
        (sum / frame.len().max(1) as f64).sqrt() as f32
    }

    /// Feed samples; returns any completed segments.
    pub fn push(&mut self, samples: &[i16]) -> Vec<Vec<i16>> {
        self.carry.extend_from_slice(samples);
        let mut done = Vec::new();
        while self.carry.len() >= FRAME {
            let frame: Vec<i16> = self.carry.drain(..FRAME).collect();
            let loud = Self::rms(&frame) >= self.cfg.rms_threshold;

            if self.speaking {
                self.seg.extend_from_slice(&frame);
                self.quiet_run = if loud { 0 } else { self.quiet_run + 1 };
                let frames = self.seg.len() / FRAME;
                if self.quiet_run >= self.cfg.hangover_frames || frames >= self.cfg.max_frames {
                    if frames >= self.cfg.min_frames {
                        done.push(std::mem::take(&mut self.seg));
                    } else {
                        self.seg.clear();
                    }
                    self.speaking = false;
                    self.loud_run = 0;
                    self.quiet_run = 0;
                    self.pre_roll.clear();
                }
            } else if loud {
                self.loud_run += 1;
                self.pre_roll.extend_from_slice(&frame);
                if self.loud_run >= self.cfg.start_frames {
                    self.speaking = true;
                    self.seg = std::mem::take(&mut self.pre_roll);
                    self.quiet_run = 0;
                }
            } else {
                self.loud_run = 0;
                self.pre_roll.clear();
            }
        }
        done
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(ms: usize) -> Vec<i16> {
        (0..SAMPLE_RATE * ms / 1000).map(|i| ((i as f32 * 0.3).sin() * 8000.0) as i16).collect()
    }
    fn silence(ms: usize) -> Vec<i16> { vec![0; SAMPLE_RATE * ms / 1000] }

    #[test]
    fn two_bursts_make_two_segments() {
        let mut s = Segmenter::new(VadConfig::default());
        let mut out = vec![];
        for part in [silence(300), tone(1000), silence(800), tone(700), silence(800)] {
            out.extend(s.push(&part));
        }
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn long_speech_is_force_cut() {
        let mut s = Segmenter::new(VadConfig::default());
        let out = s.push(&tone(12_000));
        assert!(out.len() >= 2, "max_frames should bound segment length");
    }

    #[test]
    fn blip_shorter_than_start_is_ignored() {
        let mut s = Segmenter::new(VadConfig::default());
        let mut out = s.push(&tone(20));
        out.extend(s.push(&silence(1000)));
        assert!(out.is_empty());
    }
}
