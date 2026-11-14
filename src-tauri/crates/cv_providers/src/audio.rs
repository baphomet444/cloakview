//! Platform-independent audio plumbing: whatever the OS capture layer hands us
//! (ScreenCaptureKit for system audio, cpal for the mic) goes through here.

/// Raw chunk as delivered by a capture backend. `samples` is interleaved f32.
pub struct AudioChunk {
    pub samples: Vec<f32>,
    pub channels: u16,
    pub sample_rate: u32,
}

pub fn downmix_to_mono(interleaved: &[f32], channels: u16) -> Vec<f32> {
    let ch = channels.max(1) as usize;
    if ch == 1 {
        return interleaved.to_vec();
    }
    interleaved
        .chunks_exact(ch)
        .map(|f| f.iter().sum::<f32>() / ch as f32)
        .collect()
}

/// Streaming linear resampler (keeps state across chunks, so no seams).
pub struct Resampler {
    ratio: f64, // input_rate / output_rate
    pending: Vec<f32>,
    next: f64,
}

impl Resampler {
    pub fn new(in_rate: u32, out_rate: u32) -> Self {
        Self { ratio: in_rate as f64 / out_rate as f64, pending: Vec::new(), next: 0.0 }
    }

    pub fn push(&mut self, mono: &[f32]) -> Vec<f32> {
        self.pending.extend_from_slice(mono);
        let mut out = Vec::new();
        while self.next + 1.0 < self.pending.len() as f64 {
            let i = self.next.floor() as usize;
            let f = (self.next - i as f64) as f32;
            out.push(self.pending[i] * (1.0 - f) + self.pending[i + 1] * f);
            self.next += self.ratio;
        }
        let drop = (self.next.floor() as usize).min(self.pending.len());
        self.pending.drain(..drop);
        self.next -= drop as f64;
        out
    }
}

pub fn f32_to_i16(x: &[f32]) -> Vec<i16> {
    x.iter().map(|s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stereo_downmix_averages() {
        assert_eq!(downmix_to_mono(&[1.0, 0.0, 0.5, 0.5], 2), vec![0.5, 0.5]);
    }

    #[test]
    fn resampler_48k_to_16k_is_one_third_length() {
        let mut r = Resampler::new(48_000, 16_000);
        let input: Vec<f32> = (0..4800).map(|i| (i as f32 * 0.01).sin()).collect();
        // feed in odd-sized chunks to exercise the carried state
        let mut n = 0;
        for c in input.chunks(333) {
            n += r.push(c).len();
        }
        assert!((n as i64 - 1600).abs() <= 2, "got {n}");
    }
}
