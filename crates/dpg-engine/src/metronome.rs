//! The metronome: which ticks click. The bar's first beat clicks higher,
//! the other beats normally, and the subdivisions between them quieter;
//! the sounds themselves are in `clicks`.

use drum_patterns::TimeSignature;

#[cfg(test)]
#[path = "metronome_tests.rs"]
mod tests;

/// A subdivision click's level against a beat's, in dB.
pub const SUBDIVISION_DB: f32 = -8.0;

/// Clicks a beat: the spec's 4 / 8 / 8т / 16.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Subdivision {
    /// One - the beat only.
    #[default]
    Quarter,
    /// Two.
    Eighth,
    /// Three.
    Triplet,
    /// Four.
    Sixteenth,
}

impl Subdivision {
    /// Every subdivision, fewest clicks first.
    pub const ALL: [Self; 4] = [Self::Quarter, Self::Eighth, Self::Triplet, Self::Sixteenth];

    /// Clicks a beat, 1 to 4.
    pub const fn clicks(self) -> u32 {
        self as u32 + 1
    }

    /// Its place in [`Subdivision::ALL`], to keep it in an atomic.
    pub(crate) fn index(self) -> u8 {
        self as u8
    }

    pub(crate) fn from_index(index: u8) -> Self {
        Self::ALL[usize::from(index).min(Self::ALL.len() - 1)]
    }
}

/// Which click a tick plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Click {
    /// The bar's first beat: the higher click.
    Bar,
    /// Any other beat.
    Beat,
    /// A subdivision between beats: quieter.
    Between,
}

/// The click on `tick` (counted from Start), if any. Also tells the UI
/// when Tap tempo blinks bright (a beat) or dim (a subdivision).
///
/// A subdivision the tick grid can't hold in this time signature - 16ths
/// of a 6/8 eighth - plays the beats only.
pub fn click_at(tick: u64, time: TimeSignature, subdivision: Subdivision) -> Option<Click> {
    let per_beat = u64::from(time.ticks_per_beat());
    let in_bar = tick % u64::from(time.ticks_per_bar());
    if in_bar == 0 {
        return Some(Click::Bar);
    }
    if in_bar.is_multiple_of(per_beat) {
        return Some(Click::Beat);
    }
    let clicks = u64::from(subdivision.clicks());
    (per_beat.is_multiple_of(clicks) && in_bar.is_multiple_of(per_beat / clicks))
        .then_some(Click::Between)
}
