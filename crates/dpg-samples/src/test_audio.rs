//! Generated audio and measurements shared by the tests.

use crate::Audio;

/// The test tone's pitch.
pub const TONE_HZ: f32 = 440.0;

/// `seconds` of a half-scale sine at `hz`.
pub fn sine(rate: u32, seconds: f32, hz: f32) -> Vec<f32> {
    let frames = (rate as f32 * seconds) as usize;
    (0..frames)
        .map(|i| 0.5 * (2.0 * std::f32::consts::PI * hz * i as f32 / rate as f32).sin())
        .collect()
}

/// The largest absolute sample.
pub fn peak(samples: &[f32]) -> f32 {
    samples.iter().fold(0.0, |m, s| m.max(s.abs()))
}

/// A tone's pitch from its rising zero crossings, over the middle half
/// (codecs and filters smear the edges).
pub fn pitch(audio: &Audio) -> f32 {
    let n = audio.samples.len();
    let middle = &audio.samples[n / 4..3 * n / 4];
    let rising = middle
        .windows(2)
        .filter(|w| w[0] < 0.0 && w[1] >= 0.0)
        .count();
    rising as f32 * audio.rate as f32 / middle.len() as f32
}
