//! A drum pattern: hits on the tick grid over 1 to 4 bars. The generators
//! write it; the engine plays it and the notation draws it.

use std::fmt;

use crate::time::TimeSignature;

#[cfg(test)]
#[path = "pattern_tests.rs"]
mod tests;

/// The longest pattern, in bars.
pub const MAX_BARS: u8 = 4;

/// What is hit. Ordered as on the staff, top to bottom.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Instrument {
    /// Crash cymbal.
    Crash,
    /// Hi-hat.
    HiHat,
    /// Snare drum.
    Snare,
    /// Bass drum.
    Kick,
}

impl Instrument {
    /// Every instrument, top of the staff first.
    pub const ALL: [Self; 4] = [Self::Crash, Self::HiHat, Self::Snare, Self::Kick];
}

/// How hard a hit is played. A kit may hold a separate sample for a
/// ghost; otherwise the kind sets the volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HitKind {
    /// Louder than normal.
    Accent,
    /// An ordinary hit.
    Normal,
    /// Barely there - written in parentheses.
    Ghost,
}

/// One hit: an instrument at a tick from the start of the pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hit {
    /// Ticks from the start of the pattern's first bar.
    pub tick: u32,
    /// What is hit.
    pub instrument: Instrument,
    /// How hard.
    pub kind: HitKind,
}

impl Hit {
    /// A hit of the given kind.
    pub const fn new(tick: u32, instrument: Instrument, kind: HitKind) -> Self {
        Self {
            tick,
            instrument,
            kind,
        }
    }
}

/// Why a pattern refused a change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternError {
    /// Bars must be 1 to [`MAX_BARS`].
    BarsOutOfRange(u8),
    /// The tick lies past the pattern's end.
    TickOutOfRange {
        /// The tick asked for.
        tick: u32,
        /// The pattern's length in ticks.
        len: u32,
    },
}

impl fmt::Display for PatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BarsOutOfRange(bars) => write!(f, "{bars} bars - a pattern has 1 to {MAX_BARS}"),
            Self::TickOutOfRange { tick, len } => {
                write!(f, "tick {tick} is past the pattern's end ({len} ticks)")
            }
        }
    }
}

impl std::error::Error for PatternError {}

/// Hits over 1 to 4 bars of one time signature. At most one hit per
/// instrument on a tick, kept in playing order: by tick, then from the top
/// of the staff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern {
    time: TimeSignature,
    bars: u8,
    hits: Vec<Hit>,
}

impl Pattern {
    /// An empty pattern.
    pub fn new(time: TimeSignature, bars: u8) -> Result<Self, PatternError> {
        if !(1..=MAX_BARS).contains(&bars) {
            return Err(PatternError::BarsOutOfRange(bars));
        }
        Ok(Self {
            time,
            bars,
            hits: Vec::new(),
        })
    }

    /// The time signature.
    pub fn time(&self) -> TimeSignature {
        self.time
    }

    /// Length in bars.
    pub fn bars(&self) -> u8 {
        self.bars
    }

    /// Length in ticks.
    pub fn len_ticks(&self) -> u32 {
        self.time.ticks_per_bar() * self.bars as u32
    }

    /// Every hit, in playing order.
    pub fn hits(&self) -> &[Hit] {
        &self.hits
    }

    /// One instrument's hits, in playing order.
    pub fn hits_of(&self, instrument: Instrument) -> impl Iterator<Item = &Hit> {
        self.hits.iter().filter(move |h| h.instrument == instrument)
    }

    /// The instrument's hit on this tick, if any.
    pub fn get(&self, tick: u32, instrument: Instrument) -> Option<Hit> {
        self.find(tick, instrument).ok().map(|i| self.hits[i])
    }

    /// Places a hit, replacing the instrument's hit on that tick.
    pub fn set(&mut self, hit: Hit) -> Result<(), PatternError> {
        let len = self.len_ticks();
        if hit.tick >= len {
            return Err(PatternError::TickOutOfRange {
                tick: hit.tick,
                len,
            });
        }
        match self.find(hit.tick, hit.instrument) {
            Ok(i) => self.hits[i] = hit,
            Err(i) => self.hits.insert(i, hit),
        }
        Ok(())
    }

    /// Removes the instrument's hit on this tick and returns it.
    pub fn remove(&mut self, tick: u32, instrument: Instrument) -> Option<Hit> {
        self.find(tick, instrument)
            .ok()
            .map(|i| self.hits.remove(i))
    }

    /// Removes all of one instrument's hits - before a generator rewrites
    /// that part.
    pub fn clear(&mut self, instrument: Instrument) {
        self.hits.retain(|h| h.instrument != instrument);
    }

    fn find(&self, tick: u32, instrument: Instrument) -> Result<usize, usize> {
        self.hits
            .binary_search_by_key(&(tick, instrument), |h| (h.tick, h.instrument))
    }
}
