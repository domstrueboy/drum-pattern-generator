//! The hi-hat part. A pulse is one hit every note value; Random keeps
//! the pulse's hits on the beats and some of the others at random.

use rand::seq::IndexedRandom;
use rand::{Rng, RngExt};

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
    for tick in pulse_ticks(pattern, pulse) {
        set_hat(pattern, tick);
    }
}

/// The default share of Random's off-beat hits kept, in percent: a coin
/// toss.
pub const HI_HAT_DENSITY: u8 = 50;

/// Rewrites the hi-hat as a random part on the `pulse`'s grid. Every beat
/// the pulse lands on keeps its hit; each other hit is kept with a chance
/// of `density` percent (clamped to 100), and a bar left empty keeps one
/// at random. For Random, give it a variant's RNG (see
/// [`crate::Variants`]).
pub fn hi_hat_random(pattern: &mut Pattern, pulse: NoteValue, density: u8, rng: &mut impl Rng) {
    pattern.clear(Instrument::HiHat);
    let time = pattern.time();
    let (per_bar, per_beat) = (time.ticks_per_bar(), time.ticks_per_beat());
    let chance = f64::from(density.min(100)) / 100.0;
    let grid = pulse_ticks(pattern, pulse);
    for bar in 0..pattern.bars() as u32 {
        let in_bar: Vec<u32> = grid
            .iter()
            .copied()
            .filter(|t| t / per_bar == bar)
            .collect();
        let mut kept: Vec<u32> = in_bar
            .iter()
            .copied()
            .filter(|t| (t % per_bar).is_multiple_of(per_beat) || rng.random_bool(chance))
            .collect();
        // Only a pulse that misses every beat of a bar gets here.
        if kept.is_empty()
            && let Some(&one) = in_bar.choose(rng)
        {
            kept.push(one);
        }
        for tick in kept {
            set_hat(pattern, tick);
        }
    }
}

fn pulse_ticks(pattern: &Pattern, pulse: NoteValue) -> Vec<u32> {
    (0..pattern.len_ticks())
        .step_by(pulse.ticks() as usize)
        .collect()
}

fn set_hat(pattern: &mut Pattern, tick: u32) {
    let hit = Hit::new(tick, Instrument::HiHat, HitKind::Normal);
    pattern.set(hit).expect("every tick is inside the pattern");
}
