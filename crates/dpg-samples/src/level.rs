//! Bringing an instrument's samples up to full scale together, with one
//! gain, so a ghost sample stays as much quieter than the normal one as it
//! was recorded.

use crate::decode::Audio;

#[cfg(test)]
#[path = "level_tests.rs"]
mod tests;

/// Scales `sounds` - one instrument's samples - by one gain, so the
/// loudest peak lands at full scale. All silence is left as it is.
pub fn level<'a>(sounds: impl IntoIterator<Item = &'a mut Audio>) {
    let mut sounds: Vec<&mut Audio> = sounds.into_iter().collect();
    let peak = sounds
        .iter()
        .flat_map(|a| a.samples.iter())
        .fold(0.0f32, |m, s| m.max(s.abs()));
    if peak == 0.0 {
        return;
    }
    let gain = 1.0 / peak;
    for sound in &mut sounds {
        sound.samples.iter_mut().for_each(|s| *s *= gain);
    }
}
