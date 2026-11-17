//! The kick part. Base kicks sit on 8ths the snare leaves free; extra
//! kicks, on the 16ths between, come with the ghost notes (step 2f).

use rand::Rng;
use rand::seq::IndexedRandom;

use crate::pattern::{Hit, HitKind, Instrument, Pattern};
use crate::time::NoteValue;

#[cfg(test)]
#[path = "kick_tests.rs"]
mod tests;

/// Rewrites the base kicks: `count` kicks a bar, placed at random on 8ths
/// that have no snare hit of any kind. Each bar gets its own. Asking for
/// more than the free 8ths places one on each. Kicks between the 8ths -
/// the extra kicks - and other instruments are left alone.
pub fn base_kicks(pattern: &mut Pattern, count: u8, rng: &mut impl Rng) {
    let per_bar = pattern.time().ticks_per_bar();
    let eighth = NoteValue::Eighth.ticks();
    let on_eighth = move |tick: u32| (tick % per_bar).is_multiple_of(eighth);
    pattern.retain(|h| h.instrument != Instrument::Kick || !on_eighth(h.tick));

    for bar in 0..pattern.bars() as u32 {
        let start = bar * per_bar;
        let free: Vec<u32> = (start..start + per_bar)
            .step_by(eighth as usize)
            .filter(|&t| pattern.get(t, Instrument::Snare).is_none())
            .collect();
        let chosen: Vec<u32> = free.sample(rng, count as usize).copied().collect();
        for tick in chosen {
            let hit = Hit::new(tick, Instrument::Kick, HitKind::Normal);
            pattern.set(hit).expect("every tick is inside the pattern");
        }
    }
}
