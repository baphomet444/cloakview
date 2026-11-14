//! Decode the raw buffers ScreenCaptureKit hands us into an `AudioChunk`.
//! Platform-independent so it can be unit-tested anywhere.
//!
//! ScreenCaptureKit audio is 32-bit float PCM, usually NON-interleaved
//! (one buffer per channel). The format flags tell us which.

use crate::audio::AudioChunk;

pub struct SckFormat {
    pub sample_rate: u32,
    pub channels_per_frame: u32,
    pub bits_per_channel: u32,
    pub is_float: bool,
    pub non_interleaved: bool,
}

pub fn decode_buffers(buffers: &[Vec<u8>], fmt: &SckFormat) -> Option<AudioChunk> {
    // Only f32 PCM is handled; anything else is skipped rather than guessed at.
    if !fmt.is_float || fmt.bits_per_channel != 32 || buffers.is_empty() {
        return None;
    }
    let parsed: Vec<Vec<f32>> = buffers
        .iter()
        .map(|b| b.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect())
        .collect();

    if fmt.non_interleaved && parsed.len() > 1 {
        let n = parsed.iter().map(Vec::len).min()?;
        let mut samples = Vec::with_capacity(n * parsed.len());
        for i in 0..n {
            for ch in &parsed {
                samples.push(ch[i]);
            }
        }
        Some(AudioChunk { samples, channels: parsed.len() as u16, sample_rate: fmt.sample_rate })
    } else {
        // One buffer: either interleaved N-channel, or genuinely mono.
        let channels = if fmt.non_interleaved { 1 } else { fmt.channels_per_frame.max(1) as u16 };
        Some(AudioChunk {
            samples: parsed.into_iter().next()?,
            channels,
            sample_rate: fmt.sample_rate,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes(v: &[f32]) -> Vec<u8> { v.iter().flat_map(|x| x.to_le_bytes()).collect() }
    fn fmt(non_interleaved: bool) -> SckFormat {
        SckFormat { sample_rate: 48_000, channels_per_frame: 2, bits_per_channel: 32, is_float: true, non_interleaved }
    }

    #[test]
    fn planar_stereo_is_interleaved() {
        let l = bytes(&[1.0, 2.0]);
        let r = bytes(&[-1.0, -2.0]);
        let c = decode_buffers(&[l, r], &fmt(true)).unwrap();
        assert_eq!(c.channels, 2);
        assert_eq!(c.samples, vec![1.0, -1.0, 2.0, -2.0]);
    }

    #[test]
    fn interleaved_single_buffer_keeps_channel_count() {
        let c = decode_buffers(&[bytes(&[0.1, 0.2, 0.3, 0.4])], &fmt(false)).unwrap();
        assert_eq!(c.channels, 2);
        assert_eq!(c.samples.len(), 4);
    }

    #[test]
    fn non_float_is_rejected() {
        let mut f = fmt(true);
        f.is_float = false;
        assert!(decode_buffers(&[bytes(&[0.0])], &f).is_none());
    }
}
