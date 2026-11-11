//! Sample conversion for the microphone: any device format in, 24 kHz mono 16-bit out.
//!
//! Pure functions and one small stateful resampler, so they are unit-tested directly.

/// Averages the channels of interleaved samples into one.
pub fn downmix(interleaved: &[f32], channels: usize) -> Vec<f32> {
    if channels <= 1 {
        return interleaved.to_vec();
    }
    interleaved.chunks_exact(channels).map(|frame| frame.iter().sum::<f32>() / channels as f32).collect()
}

/// 16-bit little-endian PCM, clamped.
pub fn to_s16le(samples: &[f32]) -> Vec<u8> {
    samples
        .iter()
        .flat_map(|s| ((s.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16).to_le_bytes())
        .collect()
}

/// Streaming linear-interpolation resampler. Audio arrives in blocks, so it keeps
/// its position between calls and the output is the same however the input is cut.
/// Quality is plenty for speech recognition; it is not meant for music.
#[derive(Debug)]
pub struct Resampler {
    /// Input samples consumed per output sample.
    step: f64,
    /// Where the next output sample sits, relative to the start of the next block
    /// (`-1` is the last sample of the previous block).
    pos: f64,
    last: f32,
}

impl Resampler {
    pub fn new(in_rate: u32, out_rate: u32) -> Self {
        Self { step: f64::from(in_rate.max(1)) / f64::from(out_rate.max(1)), pos: 0.0, last: 0.0 }
    }

    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        let mut out = Vec::with_capacity((input.len() as f64 / self.step) as usize + 1);
        while self.pos < input.len() as f64 {
            let i = self.pos.floor() as isize;
            // The next input sample is needed to interpolate: wait for the next block.
            if (i + 1) as usize >= input.len() {
                break;
            }
            let a = if i < 0 { self.last } else { input[i as usize] };
            let b = input[(i + 1) as usize];
            let frac = (self.pos - i as f64) as f32;
            out.push(a + (b - a) * frac);
            self.pos += self.step;
        }
        if let Some(&last) = input.last() {
            self.pos -= input.len() as f64;
            self.last = last;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stereo_is_averaged_and_mono_is_untouched() {
        assert_eq!(downmix(&[1.0, 0.0, 0.5, 0.5], 2), vec![0.5, 0.5]);
        assert_eq!(downmix(&[0.1, 0.2], 1), vec![0.1, 0.2]);
    }

    #[test]
    fn samples_become_16_bit_and_are_clamped() {
        let bytes = to_s16le(&[0.0, 0.5, -0.5, 3.0, -3.0]);
        let out: Vec<i16> = bytes.as_chunks::<2>().0.iter().map(|b| i16::from_le_bytes(*b)).collect();
        assert_eq!(out, vec![0, 16384, -16384, i16::MAX, -i16::MAX]);
    }

    fn ramp(n: usize) -> Vec<f32> {
        (0..n).map(|i| i as f32 / n as f32).collect()
    }

    #[test]
    fn halving_the_rate_halves_the_samples() {
        let mut r = Resampler::new(48_000, 24_000);
        let out = r.process(&ramp(4800));
        assert!((2398..=2400).contains(&out.len()), "got {}", out.len());
        // A ramp stays a ramp: every other input sample.
        assert!((out[10] - 20.0 / 4800.0).abs() < 1e-5);
    }

    #[test]
    fn the_output_does_not_depend_on_how_the_input_is_cut() {
        let input = ramp(4410);
        let whole = Resampler::new(44_100, 24_000).process(&input);
        let mut cut = Resampler::new(44_100, 24_000);
        let mut parts = Vec::new();
        for block in input.chunks(441) {
            parts.extend(cut.process(block));
        }
        assert_eq!(whole.len(), parts.len());
        assert!(whole.iter().zip(&parts).all(|(a, b)| (a - b).abs() < 1e-5));
    }

    #[test]
    fn the_same_rate_passes_samples_through() {
        let mut r = Resampler::new(24_000, 24_000);
        let a = r.process(&[0.1, 0.2, 0.3]);
        let b = r.process(&[0.4]);
        assert_eq!([a, b].concat(), vec![0.1, 0.2, 0.3]);
    }
}
