//! Musical time on a tick grid. Twelve ticks a quarter note is the
//! smallest grid that holds 16ths, every triplet and both dotted values
//! the spec uses.

use std::fmt;

#[cfg(test)]
#[path = "time_tests.rs"]
mod tests;

/// Ticks in a quarter note.
pub const TICKS_PER_QUARTER: u32 = 12;

/// A time signature: `beats` notes of value `unit` in a bar - 4/4, 3/4,
/// 6/8 and so on. The app starts with 4/4; the model doesn't assume it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimeSignature {
    beats: u8,
    unit: u8,
}

impl TimeSignature {
    /// Common time.
    pub const FOUR_FOUR: Self = Self { beats: 4, unit: 4 };

    /// `None` unless `beats` is 1 to 16 and `unit` is 2, 4, 8 or 16 - the
    /// units the grid divides into whole ticks.
    pub const fn new(beats: u8, unit: u8) -> Option<Self> {
        let unit_ok = matches!(unit, 2 | 4 | 8 | 16);
        if beats >= 1 && beats <= 16 && unit_ok {
            Some(Self { beats, unit })
        } else {
            None
        }
    }

    /// Beats in a bar - the top number.
    pub const fn beats(self) -> u8 {
        self.beats
    }

    /// The note value of one beat - the bottom number.
    pub const fn unit(self) -> u8 {
        self.unit
    }

    /// Ticks in one beat: 12 for a quarter, 6 for an 8th.
    pub const fn ticks_per_beat(self) -> u32 {
        TICKS_PER_QUARTER * 4 / self.unit as u32
    }

    /// Ticks in one bar: 48 in 4/4, 36 in 3/4 and 6/8.
    pub const fn ticks_per_bar(self) -> u32 {
        self.ticks_per_beat() * self.beats as u32
    }
}

impl Default for TimeSignature {
    fn default() -> Self {
        Self::FOUR_FOUR
    }
}

impl fmt::Display for TimeSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.beats, self.unit)
    }
}

/// The note values the spec uses - the hi-hat pulses and the notation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoteValue {
    /// 4.
    DottedQuarter,
    /// 4
    Quarter,
    /// 8.
    DottedEighth,
    /// 4т
    QuarterTriplet,
    /// 8
    Eighth,
    /// 8т
    EighthTriplet,
    /// 16
    Sixteenth,
    /// 16т
    SixteenthTriplet,
}

impl NoteValue {
    /// Every value, longest first.
    pub const ALL: [Self; 8] = [
        Self::DottedQuarter,
        Self::Quarter,
        Self::DottedEighth,
        Self::QuarterTriplet,
        Self::Eighth,
        Self::EighthTriplet,
        Self::Sixteenth,
        Self::SixteenthTriplet,
    ];

    /// Length in ticks.
    pub const fn ticks(self) -> u32 {
        match self {
            Self::DottedQuarter => 18,
            Self::Quarter => 12,
            Self::DottedEighth => 9,
            Self::QuarterTriplet => 8,
            Self::Eighth => 6,
            Self::EighthTriplet => 4,
            Self::Sixteenth => 3,
            Self::SixteenthTriplet => 2,
        }
    }

    /// Whether it's written as a triplet - which also picks its konnakol.
    pub const fn is_triplet(self) -> bool {
        matches!(
            self,
            Self::QuarterTriplet | Self::EighthTriplet | Self::SixteenthTriplet
        )
    }
}
