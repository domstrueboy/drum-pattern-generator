//! Konnakol under the notes: a syllable on every tick that has a note,
//! nothing elsewhere. Straight notes take та-ка by their 16th, triplets
//! take та-ки-та along each group of three.

use crate::pattern::Pattern;

#[cfg(test)]
#[path = "konnakol_tests.rs"]
mod tests;

/// A konnakol syllable. The spec writes each by its first letter:
/// `т`, `к`, `к`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Syllable {
    /// "та" - a straight note on an 8th, or a triplet group's first or
    /// last note.
    Ta,
    /// "ка" - a straight 16th between 8ths.
    Ka,
    /// "ки" - the middle of a triplet group.
    Ki,
}

/// One syllable and where it sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KonnakolMark {
    /// Ticks from the start of the pattern.
    pub tick: u32,
    /// What is said there.
    pub syllable: Syllable,
}

/// A 16th, in ticks: straight notes sit on this grid.
const SIXTEENTH: u32 = 3;
/// A quarter triplet, in ticks: its group of three spans two quarters.
const QUARTER_TRIPLET: u32 = 8;

/// The konnakol for every tick that has a note, of any instrument.
///
/// A note on the 16th grid is straight: `Ta` on an 8th, `Ka` between.
/// Dotted values land there too. Any other note is a triplet; which
/// triplet is read from its beat - the coarsest triplet grid that holds
/// all the beat's triplet notes - so a 16th triplet's third note is `Ta`
/// while an 8th triplet's second is `Ki`. A pair of quarters whose only
/// off-grid notes are the 4т pulse's (8 and 16 ticks in) is read as one
/// quarter-triplet group.
pub fn konnakol(pattern: &Pattern) -> Vec<KonnakolMark> {
    let mut ticks: Vec<u32> = pattern.hits().iter().map(|h| h.tick).collect();
    ticks.dedup(); // hits are in tick order, so equal ticks are neighbours
    let reader = Reader {
        per_beat: pattern.time().ticks_per_beat(),
        per_bar: pattern.time().ticks_per_bar(),
        ticks: &ticks,
    };
    ticks
        .iter()
        .map(|&tick| KonnakolMark {
            tick,
            syllable: reader.syllable(tick),
        })
        .collect()
}

struct Reader<'a> {
    per_beat: u32,
    per_bar: u32,
    ticks: &'a [u32],
}

impl Reader<'_> {
    fn syllable(&self, tick: u32) -> Syllable {
        let in_bar = tick % self.per_bar;
        if in_bar.is_multiple_of(SIXTEENTH) {
            return if in_bar.is_multiple_of(2 * SIXTEENTH) {
                Syllable::Ta
            } else {
                Syllable::Ka
            };
        }
        if let Some(start) = self.quarter_triplet_group(tick) {
            return triplet_syllable(tick - start, QUARTER_TRIPLET);
        }
        let beat_start = tick - in_bar % self.per_beat;
        let off_grid: Vec<u32> = self
            .off_grid_in(beat_start, beat_start + self.per_beat)
            .map(|t| t - beat_start)
            .collect();
        // The coarsest triplet grid whose groups fill the beat and hold
        // every off-grid note in it. A step of 1 always fits.
        let step = [8, 4, 2, 1]
            .into_iter()
            .find(|&s| {
                self.per_beat.is_multiple_of(3 * s) && off_grid.iter().all(|t| t.is_multiple_of(s))
            })
            .unwrap_or(1);
        triplet_syllable(tick - beat_start, step)
    }

    /// The start of the two-quarter span this tick's quarter-triplet group
    /// covers, if its notes read as one.
    fn quarter_triplet_group(&self, tick: u32) -> Option<u32> {
        const SPAN: u32 = 3 * QUARTER_TRIPLET;
        if self.per_beat != 12 {
            return None;
        }
        let in_bar = tick % self.per_bar;
        let start = tick - in_bar % SPAN;
        if in_bar - in_bar % SPAN + SPAN > self.per_bar {
            return None; // the span would run past the bar, as in 3/4's last beat
        }
        let off_grid: Vec<u32> = self
            .off_grid_in(start, start + SPAN)
            .map(|t| t - start)
            .collect();
        (off_grid == [QUARTER_TRIPLET, 2 * QUARTER_TRIPLET]).then_some(start)
    }

    /// The ticks in `from..to` that are off the 16th grid.
    fn off_grid_in(&self, from: u32, to: u32) -> impl Iterator<Item = u32> + '_ {
        let per_bar = self.per_bar;
        self.ticks
            .iter()
            .copied()
            .filter(move |&t| t >= from && t < to && !(t % per_bar).is_multiple_of(SIXTEENTH))
    }
}

/// The syllable `offset` ticks into a run of triplet groups of `step`.
fn triplet_syllable(offset: u32, step: u32) -> Syllable {
    if (offset / step) % 3 == 1 {
        Syllable::Ki
    } else {
        Syllable::Ta
    }
}
