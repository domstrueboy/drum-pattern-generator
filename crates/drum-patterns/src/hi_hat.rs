//! The hi-hat part. A pulse is one hit every note value; Random keeps
//! the pulse's hits on the beats and some of the others at random.
//! Accents go on the beats or on the и, wherever the hits land on them.

use rand::seq::IndexedRandom;
use rand::{Rng, RngExt};

use crate::pattern::{Hit, HitKind, Instrument, Pattern};
use crate::time::NoteValue;

#[cfg(test)]
#[path = "hi_hat_tests.rs"]
mod tests;

/// Rewrites the hi-hat as one hit every `pulse`. A pulse that doesn't
/// divide the bar runs on across the barline, landing somewhere new in
/// each bar - in 4/4 a dotted pulse lines up again after 3 bars - and on
/// across loops of the pattern: `lap` is which time through the pattern
/// this is, from 0 (see [`hi_hat_laps`]). Other instruments are left
/// alone.
pub fn hi_hat_pulse(pattern: &mut Pattern, pulse: NoteValue, lap: u32) {
    pattern.clear(Instrument::HiHat);
    for tick in pulse_ticks(pattern, pulse, lap) {
        set_hat(pattern, tick);
    }
}

/// How many times a pattern loops before `pulse` lines up with its start
/// again: 1 for a pulse that divides the pattern, up to 3 for a dotted
/// one. Lap `n` of the hi-hat is the same as lap `n` modulo this.
pub fn hi_hat_laps(pattern: &Pattern, pulse: NoteValue) -> u32 {
    let (len, step) = (pattern.len_ticks(), pulse.ticks());
    step / gcd(len, step)
}

/// The default share of Random's off-beat hits kept, in percent: a coin
/// toss.
pub const HI_HAT_DENSITY: u8 = 50;

/// Rewrites the hi-hat as a random part on the `pulse`'s grid. Every beat
/// the pulse lands on keeps its hit; each other hit is kept with a chance
/// of `density` percent (clamped to 100), and a bar left empty keeps one
/// at random. `lap` is as for [`hi_hat_pulse`]; each lap is random on its
/// own grid. For Random, give it a variant's RNG (see
/// [`crate::Variants`]).
pub fn hi_hat_random(
    pattern: &mut Pattern,
    pulse: NoteValue,
    density: u8,
    lap: u32,
    rng: &mut impl Rng,
) {
    pattern.clear(Instrument::HiHat);
    let time = pattern.time();
    let (per_bar, per_beat) = (time.ticks_per_bar(), time.ticks_per_beat());
    let chance = f64::from(density.min(100)) / 100.0;
    let grid = pulse_ticks(pattern, pulse, lap);
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

/// Which hi-hat hits are accented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HiHatAccents {
    /// None: every hit is played the same.
    #[default]
    Off,
    /// The hits on the beats: `1 2 3 4`.
    Beats,
    /// The hits on the и between the beats, as in disco or funk.
    Offbeats,
}

/// Accents the hi-hat's hits that land where `accents` says, and makes
/// the others normal. A pulse that never lands there - 8th triplets on
/// the и - gets no accents. Other instruments are left alone.
pub fn hi_hat_accents(pattern: &mut Pattern, accents: HiHatAccents) {
    let per_beat = pattern.time().ticks_per_beat();
    let accented = |tick: u32| match accents {
        HiHatAccents::Off => false,
        HiHatAccents::Beats => tick.is_multiple_of(per_beat),
        HiHatAccents::Offbeats => tick % per_beat == per_beat / 2,
    };
    let hats: Vec<Hit> = pattern.hits_of(Instrument::HiHat).copied().collect();
    for hat in hats {
        let kind = if accented(hat.tick) {
            HitKind::Accent
        } else {
            HitKind::Normal
        };
        let hit = Hit::new(hat.tick, Instrument::HiHat, kind);
        pattern
            .set(hit)
            .expect("the hit was already in the pattern");
    }
}

/// The pulse's ticks in the pattern on its `lap`th time through: the
/// first falls where the pulse, running on from the pattern's first
/// start, next lands.
fn pulse_ticks(pattern: &Pattern, pulse: NoteValue, lap: u32) -> Vec<u32> {
    let (len, step) = (pattern.len_ticks(), pulse.ticks());
    let behind = (u64::from(lap) * u64::from(len) % u64::from(step)) as u32;
    let first = (step - behind) % step;
    (first..len).step_by(step as usize).collect()
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn set_hat(pattern: &mut Pattern, tick: u32) {
    let hit = Hit::new(tick, Instrument::HiHat, HitKind::Normal);
    pattern.set(hit).expect("every tick is inside the pattern");
}
