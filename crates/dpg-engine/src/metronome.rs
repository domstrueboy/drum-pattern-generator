//! The metronome: which ticks click, and the clicks themselves. The bar's
//! first beat clicks higher, the other beats normally, and the
//! subdivisions between them quieter.

use std::f32::consts::TAU;

use drum_patterns::TimeSignature;

#[cfg(test)]
#[path = "metronome_tests.rs"]
mod tests;

/// A subdivision click's level against a beat's, in dB.
pub const SUBDIVISION_DB: f32 = -8.0;
/// How long a click lasts.
pub const CLICK_MS: u32 = 40;
/// The bar's first beat.
const HIGH_HZ: f32 = 2_000.0;
/// Every other click.
const LOW_HZ: f32 = 1_000.0;
/// How fast a click dies away: to -43 dB at 30 ms.
const DECAY_MS: f32 = 6.0;

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

/// The click sounds, generated at the output's rate. One sound for now;
/// the five of the Sound button come in step 6.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Clicks {
    high: Vec<f32>,
    low: Vec<f32>,
}

impl Clicks {
    pub(crate) fn new(rate: u32) -> Self {
        Self {
            high: tone(rate, HIGH_HZ),
            low: tone(rate, LOW_HZ),
        }
    }

    /// What `click` plays: a sample and its gain.
    pub(crate) fn sound(&self, click: Click) -> (&[f32], f32) {
        match click {
            Click::Bar => (&self.high, 1.0),
            Click::Beat => (&self.low, 1.0),
            Click::Between => (&self.low, 10f32.powf(SUBDIVISION_DB / 20.0)),
        }
    }
}

/// A sine burst that dies away fast: heard as a click with a pitch.
fn tone(rate: u32, hz: f32) -> Vec<f32> {
    let rate = rate as f32;
    let frames = (rate * CLICK_MS as f32 / 1000.0) as usize;
    let decay = rate * DECAY_MS / 1000.0;
    (0..frames)
        .map(|i| (TAU * hz * i as f32 / rate).sin() * (-(i as f32) / decay).exp())
        .collect()
}
