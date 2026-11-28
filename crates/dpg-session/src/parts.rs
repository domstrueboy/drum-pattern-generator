//! The pattern's parts as the submenus set them. Each part keeps its own
//! seed, so making one part again leaves the others where they are - only
//! the parts placed around it may move.

use drum_patterns::{
    EXTRA_KICK_LEVELS, Feel, GHOST_LEVELS, HI_HAT_DENSITY, HiHatAccents, MAX_BARS, NoteValue,
    Pattern, TimeSignature, Variants, base_kicks, extra_kicks, ghost_notes, hi_hat_accents,
    hi_hat_laps, hi_hat_pulse, hi_hat_random, snare,
};
use rand::rngs::StdRng;
use rand::{Rng, RngExt, SeedableRng};

#[cfg(test)]
#[path = "parts_tests.rs"]
mod tests;

/// The most of any counted hit a bar: one on every 16th.
pub const MAX_COUNT: u8 = 16;

/// How many variants Random holds until the user picks another count.
pub const RANDOM_COUNT: u8 = 4;

/// Random's set: a seed, how many variants, and the one playing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Set {
    seed: u64,
    count: u8,
    index: u8,
}

impl Set {
    fn new(seed: u64) -> Self {
        Self {
            seed,
            count: RANDOM_COUNT,
            index: 0,
        }
    }

    /// How many variants the set holds.
    pub fn count(&self) -> u8 {
        self.count
    }

    /// The variant playing, from 0.
    pub fn index(&self) -> u8 {
        self.index
    }

    fn variants(&self) -> Variants {
        Variants::new(self.seed, self.count)
    }

    fn step(&mut self) {
        self.index = self.variants().next(self.index);
    }

    fn set_count(&mut self, count: u8) {
        self.count = Variants::new(self.seed, count).count();
        self.index = 0;
    }

    fn rng(&self) -> StdRng {
        self.variants().rng(self.index)
    }
}

/// Generate's ladder: a count a bar, and a level each press raises.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ladder {
    count: u8,
    level: u8,
}

impl Ladder {
    fn new(count: u8) -> Self {
        Self { count, level: 1 }
    }

    /// Hits a bar.
    pub fn count(&self) -> u8 {
        self.count
    }

    /// The difficulty, from 1.
    pub fn level(&self) -> u8 {
        self.level
    }

    /// A new count starts the ladder again from the easiest level.
    fn set_count(&mut self, count: u8) {
        *self = Self::new(count.min(MAX_COUNT));
    }

    fn raise(&mut self, top: u8) {
        self.level = (self.level + 1).min(top);
    }
}

/// Every part's settings and seed. The same parts always build the same
/// laps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parts {
    pub(crate) time: TimeSignature,
    pub(crate) bars: u8,
    pub(crate) pulse: NoteValue,
    pub(crate) hi_hat_random: bool,
    pub(crate) density: u8,
    pub(crate) hi_hat_accents: HiHatAccents,
    pub(crate) hi_hat_set: Set,
    pub(crate) feel: Feel,
    pub(crate) accents: u8,
    snare_seed: u64,
    pub(crate) base_kicks: u8,
    base_kick_seed: u64,
    pub(crate) extra_kicks: Ladder,
    extra_kick_seed: u64,
    pub(crate) ghosts: Ladder,
    pub(crate) ghost_set: Set,
}

impl Parts {
    /// One bar of 4/4 - 8ths on the hi-hat, the basic snare, two kicks -
    /// with every seed drawn from `rng`.
    pub fn new(rng: &mut impl Rng) -> Self {
        Self {
            time: TimeSignature::FOUR_FOUR,
            bars: 1,
            pulse: NoteValue::Eighth,
            hi_hat_random: false,
            density: HI_HAT_DENSITY,
            hi_hat_accents: HiHatAccents::Off,
            hi_hat_set: Set::new(rng.random()),
            feel: Feel::Basic,
            accents: 0,
            snare_seed: rng.random(),
            base_kicks: 2,
            base_kick_seed: rng.random(),
            extra_kicks: Ladder::new(0),
            extra_kick_seed: rng.random(),
            ghosts: Ladder::new(0),
            ghost_set: Set::new(rng.random()),
        }
    }

    /// The pattern's length, 1 to [`MAX_BARS`].
    pub fn set_bars(&mut self, bars: u8) {
        self.bars = bars.clamp(1, MAX_BARS);
    }

    /// A pulse plays in full: choosing one turns Random off.
    pub fn set_pulse(&mut self, pulse: NoteValue) {
        self.pulse = pulse;
        self.hi_hat_random = false;
    }

    /// The hi-hat's Random: the first press turns it on, each after steps
    /// to the next variant.
    pub fn hi_hat_random(&mut self) {
        if self.hi_hat_random {
            self.hi_hat_set.step();
        } else {
            self.hi_hat_random = true;
        }
    }

    /// How many variants the hi-hat's Random holds; back to the first.
    pub fn set_hi_hat_random_count(&mut self, count: u8) {
        self.hi_hat_set.set_count(count);
    }

    /// The share of Random's off-beat hits kept, in percent.
    pub fn set_density(&mut self, percent: u8) {
        self.density = percent.min(100);
    }

    /// The snare's main hits.
    pub fn set_feel(&mut self, feel: Feel) {
        self.feel = feel;
    }

    /// Accents a bar, placed afresh even when the count is the same - so
    /// choosing a count again is how to hear other accents.
    pub fn set_accents(&mut self, count: u8, rng: &mut impl Rng) {
        self.accents = count.min(MAX_COUNT);
        self.snare_seed = rng.random();
    }

    /// Ghost notes a bar, from the easiest level again.
    pub fn set_ghosts(&mut self, count: u8) {
        self.ghosts.set_count(count);
    }

    /// Generate: ghost notes a level harder, a new set of variants.
    pub fn generate_ghosts(&mut self, rng: &mut impl Rng) {
        self.ghosts.raise(GHOST_LEVELS);
        self.ghost_set = Set {
            count: self.ghost_set.count,
            ..Set::new(rng.random())
        };
    }

    /// The ghost notes' Random: the next variant at the same level.
    pub fn ghost_random(&mut self) {
        self.ghost_set.step();
    }

    /// How many variants the ghost notes' Random holds; back to the first.
    pub fn set_ghost_random_count(&mut self, count: u8) {
        self.ghost_set.set_count(count);
    }

    /// Base kicks a bar, where they were as far as the count allows.
    pub fn set_base_kicks(&mut self, count: u8) {
        self.base_kicks = count.min(MAX_COUNT);
    }

    /// Generate: the base kicks placed afresh.
    pub fn generate_base_kicks(&mut self, rng: &mut impl Rng) {
        self.base_kick_seed = rng.random();
    }

    /// Extra kicks a bar, from the easiest level again.
    pub fn set_extra_kicks(&mut self, count: u8) {
        self.extra_kicks.set_count(count);
    }

    /// Generate: extra kicks a level harder, placed afresh.
    pub fn generate_extra_kicks(&mut self, rng: &mut impl Rng) {
        self.extra_kicks.raise(EXTRA_KICK_LEVELS);
        self.extra_kick_seed = rng.random();
    }

    /// The pattern on each time through until the hi-hat lines up again -
    /// one for most pulses, up to three for a dotted one. Built in the
    /// generator's order: snare, base kicks, extra kicks, ghost notes,
    /// hi-hat, each fitting around those before it.
    pub fn build_laps(&self) -> Vec<Pattern> {
        let seeded = StdRng::seed_from_u64;
        let mut base = Pattern::new(self.time, self.bars).expect("bars are kept in range");
        snare(
            &mut base,
            self.feel,
            self.accents,
            &mut seeded(self.snare_seed),
        );
        base_kicks(&mut base, self.base_kicks, &mut seeded(self.base_kick_seed));
        extra_kicks(
            &mut base,
            self.extra_kicks.count,
            self.extra_kicks.level,
            &mut seeded(self.extra_kick_seed),
        );
        ghost_notes(
            &mut base,
            self.ghosts.count,
            self.ghosts.level,
            &mut self.ghost_set.rng(),
        );

        // One variant's laps draw in turn from one source.
        let mut rng = self.hi_hat_set.rng();
        (0..hi_hat_laps(&base, self.pulse))
            .map(|lap| {
                let mut pattern = base.clone();
                if self.hi_hat_random {
                    hi_hat_random(&mut pattern, self.pulse, self.density, lap, &mut rng);
                } else {
                    hi_hat_pulse(&mut pattern, self.pulse, lap);
                }
                hi_hat_accents(&mut pattern, self.hi_hat_accents);
                pattern
            })
            .collect()
    }
}
