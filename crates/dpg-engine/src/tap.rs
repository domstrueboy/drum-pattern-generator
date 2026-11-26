//! Tap tempo: the BPM from the time between taps. Averages the last few,
//! so a steady hand settles on the tempo; a long pause or a clear change
//! starts counting afresh.

use std::collections::VecDeque;

use crate::clock::{BPM_MAX, BPM_MIN};

#[cfg(test)]
#[path = "tap_tests.rs"]
mod tests;

/// A pause this long, in seconds, starts a new count - longer than a beat
/// at the slowest tempo.
pub const TAP_TIMEOUT: f64 = 2.5;
/// Gaps between taps averaged.
const AVERAGED: usize = 4;
/// A gap this far off the average (as a fraction) is a new tempo, not an
/// unsteady tap.
const CHANGE: f64 = 0.4;

/// Turns taps into a tempo.
#[derive(Debug, Clone, Default)]
pub struct TapTempo {
    last: Option<f64>,
    gaps: VecDeque<f64>,
}

impl TapTempo {
    /// No taps yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// A tap at `seconds` on any steady clock. From the second tap on,
    /// returns the tempo in whole beats a minute, in the engine's range.
    pub fn tap(&mut self, seconds: f64) -> Option<f64> {
        let last = self.last.replace(seconds)?;
        let gap = seconds - last;
        if gap <= 0.0 || gap > TAP_TIMEOUT {
            self.gaps.clear();
            return None;
        }
        if let Some(average) = self.average()
            && (gap - average).abs() > average * CHANGE
        {
            self.gaps.clear();
        }
        if self.gaps.len() == AVERAGED {
            self.gaps.pop_front();
        }
        self.gaps.push_back(gap);
        let average = self.average()?;
        Some((60.0 / average).round().clamp(BPM_MIN, BPM_MAX))
    }

    fn average(&self) -> Option<f64> {
        (!self.gaps.is_empty()).then(|| self.gaps.iter().sum::<f64>() / self.gaps.len() as f64)
    }
}
