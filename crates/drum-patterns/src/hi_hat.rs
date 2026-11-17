//! The hi-hat part. A pulse is one hit every note value; Random comes
//! later (step 2g).

use crate::pattern::{Hit, HitKind, Instrument, Pattern};
use crate::time::NoteValue;

#[cfg(test)]
#[path = "hi_hat_tests.rs"]
mod tests;

/// Rewrites the hi-hat as one hit every `pulse` from the start of the
/// pattern. A pulse that doesn't divide the bar runs on across the
/// barline, landing somewhere new in each bar - in 4/4 a dotted pulse
/// lines up again after 3 bars. Other instruments are left alone.
pub fn hi_hat_pulse(pattern: &mut Pattern, pulse: NoteValue) {
    pattern.clear(Instrument::HiHat);
    for tick in (0..pattern.len_ticks()).step_by(pulse.ticks() as usize) {
        let hit = Hit::new(tick, Instrument::HiHat, HitKind::Normal);
        pattern.set(hit).expect("every tick is inside the pattern");
    }
}
