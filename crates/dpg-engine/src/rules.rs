//! Playback rules, counted in bars of playback rather than of the
//! pattern: bar skip and the crash. The engine plays by them; the UI asks
//! the same rules what was played, to light the kit up.

#[cfg(test)]
#[path = "rules_tests.rs"]
mod tests;

/// Bar skip: the pattern plays for some bars, then is silent for some,
/// while the metronome and the time keep going.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BarSkip {
    play: u8,
    skip: u8,
}

impl BarSkip {
    /// `play` bars heard, then `skip` silent - the menu's `2-1` is
    /// `new(2, 1)`. `None` unless both are at least 1.
    pub const fn new(play: u8, skip: u8) -> Option<Self> {
        if play == 0 || skip == 0 {
            None
        } else {
            Some(Self { play, skip })
        }
    }

    /// Bars heard in each round.
    pub const fn play(self) -> u8 {
        self.play
    }

    /// Bars silent in each round.
    pub const fn skip(self) -> u8 {
        self.skip
    }

    /// The menu's invert button: `2-1` becomes `1-2`.
    pub const fn inverted(self) -> Self {
        Self {
            play: self.skip,
            skip: self.play,
        }
    }

    /// Whether the pattern is heard in `bar` of playback, from 0.
    pub fn plays(self, bar: u64) -> bool {
        bar % (u64::from(self.play) + u64::from(self.skip)) < u64::from(self.play)
    }

    /// Whether `skip` lets the pattern be heard in `bar`; off plays
    /// every bar.
    pub fn allows(skip: Option<Self>, bar: u64) -> bool {
        skip.is_none_or(|s| s.plays(bar))
    }

    pub(crate) fn pack(skip: Option<Self>) -> u16 {
        skip.map_or(0, |s| u16::from(s.play) << 8 | u16::from(s.skip))
    }

    pub(crate) fn unpack(bits: u16) -> Option<Self> {
        Self::new((bits >> 8) as u8, bits as u8)
    }
}

/// Where the crash plays: on the first beat of a bar of playback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CrashRule {
    /// Once, as playing starts.
    #[default]
    Once,
    /// On the first bar and every this many bars after: 4 is every
    /// квадрат, 8 every two.
    Every(u8),
}

impl CrashRule {
    /// Whether the crash plays at the start of `bar` of playback, from 0.
    pub fn plays(self, bar: u64) -> bool {
        match self {
            Self::Once => bar == 0,
            Self::Every(bars) => bar.is_multiple_of(u64::from(bars.max(1))),
        }
    }

    pub(crate) fn pack(self) -> u16 {
        match self {
            Self::Once => 0,
            Self::Every(bars) => 0x100 | u16::from(bars),
        }
    }

    pub(crate) fn unpack(bits: u16) -> Self {
        if bits & 0x100 == 0 {
            Self::Once
        } else {
            Self::Every(bits as u8)
        }
    }
}
