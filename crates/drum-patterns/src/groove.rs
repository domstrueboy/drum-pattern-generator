//! A groove: every setting that shapes a pattern, plus one seed, built
//! into a whole pattern in one go. The looper asks for grooves this way;
//! the app builds its first pattern so, then regenerates parts by hand.

use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::ghost_notes::ghost_notes;
use crate::hi_hat::{HiHatAccents, hi_hat_accents, hi_hat_laps, hi_hat_pulse, hi_hat_random};
use crate::kick::{base_kicks, extra_kicks};
use crate::pattern::{Pattern, PatternError};
use crate::snare::{Feel, snare};
use crate::time::{NoteValue, TimeSignature};
use crate::variants::mix;

#[cfg(test)]
#[path = "groove_tests.rs"]
mod tests;

/// The settings of a groove. The same groove always builds the same
/// pattern - with the same version of `rand`, whose generator may change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Groove {
    /// The time signature.
    pub time: TimeSignature,
    /// The pattern's length, 1 to [`crate::MAX_BARS`].
    pub bars: u8,
    /// The hi-hat pulse.
    pub hi_hat: NoteValue,
    /// `None` plays the whole pulse; `Some(density)` plays Random at that
    /// density (see [`hi_hat_random`]).
    pub hi_hat_random: Option<u8>,
    /// Which hi-hat hits are accented.
    pub hi_hat_accents: HiHatAccents,
    /// The snare's main hits.
    pub feel: Feel,
    /// Snare accents a bar.
    pub accents: u8,
    /// Ghost notes a bar.
    pub ghosts: u8,
    /// The ghost notes' difficulty, 1 to [`crate::GHOST_LEVELS`].
    pub ghost_level: u8,
    /// Base kicks a bar, on the 8ths.
    pub base_kicks: u8,
    /// Extra kicks a bar, on the 16ths between.
    pub extra_kicks: u8,
    /// The extra kicks' difficulty, 1 to [`crate::EXTRA_KICK_LEVELS`].
    pub extra_kick_level: u8,
    /// Decides every random choice; a new seed is a new groove.
    pub seed: u64,
}

impl Default for Groove {
    /// One bar of 4/4: 8ths on the hi-hat, the basic snare, two kicks.
    fn default() -> Self {
        Self {
            time: TimeSignature::FOUR_FOUR,
            bars: 1,
            hi_hat: NoteValue::Eighth,
            hi_hat_random: None,
            hi_hat_accents: HiHatAccents::Off,
            feel: Feel::Basic,
            accents: 0,
            ghosts: 0,
            ghost_level: 1,
            base_kicks: 2,
            extra_kicks: 0,
            extra_kick_level: 1,
            seed: 0,
        }
    }
}

impl Groove {
    /// Builds the pattern: snare, base kicks, extra kicks, ghost notes,
    /// then hi-hat, each part fitting around those before it. Each part
    /// draws from its own seed, so a part's count only moves the parts
    /// that avoid it.
    pub fn build(&self) -> Result<Pattern, PatternError> {
        self.build_lap(0)
    }

    /// The pattern on each time through until the hi-hat lines up with it
    /// again: one pattern for most grooves, up to three when a dotted
    /// pulse runs on across loops. They differ only in the hi-hat; the
    /// first is [`Groove::build`]'s.
    pub fn build_laps(&self) -> Result<Vec<Pattern>, PatternError> {
        let first = self.build_lap(0)?;
        let laps = hi_hat_laps(&first, self.hi_hat);
        let mut all = vec![first];
        for lap in 1..laps {
            all.push(self.build_lap(lap)?);
        }
        Ok(all)
    }

    fn build_lap(&self, lap: u32) -> Result<Pattern, PatternError> {
        let mut pattern = Pattern::new(self.time, self.bars)?;
        let rng = |part: u64| StdRng::seed_from_u64(mix(self.seed, part));

        snare(&mut pattern, self.feel, self.accents, &mut rng(0));
        base_kicks(&mut pattern, self.base_kicks, &mut rng(1));
        extra_kicks(
            &mut pattern,
            self.extra_kicks,
            self.extra_kick_level,
            &mut rng(2),
        );
        ghost_notes(&mut pattern, self.ghosts, self.ghost_level, &mut rng(3));
        match self.hi_hat_random {
            None => hi_hat_pulse(&mut pattern, self.hi_hat, lap),
            // Lap 0 draws from part 4; later laps from seeds past every
            // part's.
            Some(density) => {
                let part = 4 + 8 * u64::from(lap);
                hi_hat_random(&mut pattern, self.hi_hat, density, lap, &mut rng(part));
            }
        }
        hi_hat_accents(&mut pattern, self.hi_hat_accents);
        Ok(pattern)
    }
}
