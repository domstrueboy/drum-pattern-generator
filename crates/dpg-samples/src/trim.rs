//! Cutting the silence around a hit. Silence before it makes a drum sound
//! late - a home recording's 30 ms, or the padding MP3 and AAC add; after
//! it, it only takes memory.

use crate::decode::Audio;

#[cfg(test)]
#[path = "trim_tests.rs"]
mod tests;

/// The hit starts at the first sample this far below the peak (-40 dB).
const ONSET: f32 = 0.01;
/// The ring ends after the last sample this far below the peak (-60 dB);
/// a cut there is too small a step to hear.
const TAIL: f32 = 0.001;
/// Kept before the onset, faded in, so the attack stays whole.
const PRE_ROLL_MS: u32 = 1;

/// `audio` from just before its hit to the end of its ring. Levels are
/// judged against the sound's own peak, so a quiet ghost trims like a loud
/// hit. `None` when it's all silence.
pub fn trim(audio: Audio) -> Option<Audio> {
    let peak = audio.samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    if peak == 0.0 {
        return None;
    }
    let loud = |level: f32| move |s: &f32| s.abs() >= peak * level;
    let onset = audio.samples.iter().position(loud(ONSET))?;
    let end = audio.samples.iter().rposition(loud(TAIL))? + 1;

    let pre_roll = (audio.rate * PRE_ROLL_MS / 1000) as usize;
    let start = onset.saturating_sub(pre_roll);
    let mut samples = audio.samples[start..end].to_vec();
    fade_in(&mut samples[..onset - start]);
    Some(Audio {
        rate: audio.rate,
        samples,
    })
}

/// A straight-line fade from silence across `samples`.
fn fade_in(samples: &mut [f32]) {
    let n = samples.len() as f32;
    for (i, s) in samples.iter_mut().enumerate() {
        *s *= i as f32 / n;
    }
}
