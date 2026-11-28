//! The playhead: where the notation shows the sound to be. The engine
//! reports the heard tick once an audio block; between reports the line
//! runs on at the tempo, so it glides at the screen's rate instead of
//! stepping a block at a time.

use serde::Serialize;

#[cfg(test)]
#[path = "playhead_tests.rs"]
mod tests;

/// Where the sound is in the notation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Playhead {
    /// Which lap of the pattern is playing.
    pub lap: usize,
    /// The tick within it, between whole ticks while gliding.
    pub tick: f64,
}

/// How long the line runs on without a new report, in seconds. Past it
/// the audio has stalled, and the line waits for it.
pub const MAX_GLIDE: f64 = 0.1;

/// Carries the engine's position on between its reports.
#[derive(Debug, Default)]
pub struct Glide {
    /// The latest report and when it was first seen.
    report: Option<(f64, f64)>,
    /// The tick shown last.
    shown: f64,
}

impl Glide {
    /// The tick to show at `now`, in seconds on a steady clock, given the
    /// engine's latest report and the tempo in ticks a second; `None`
    /// while stopped. Never moves back while playing: a report seen late
    /// would otherwise pull the line back a little. A report behind the
    /// last one is a new Start - the screen doesn't ask while stopped, so
    /// the stop itself may never be seen.
    pub fn at(&mut self, heard: Option<f64>, now: f64, ticks_per_second: f64) -> Option<f64> {
        let Some(heard) = heard else {
            self.report = None;
            return None;
        };
        let since = match self.report {
            Some((tick, seen)) if tick == heard => now - seen,
            Some((tick, _)) if heard > tick => {
                self.report = Some((heard, now));
                0.0
            }
            _ => {
                self.report = Some((heard, now));
                self.shown = heard;
                0.0
            }
        };
        let tick = heard + since.clamp(0.0, MAX_GLIDE) * ticks_per_second;
        self.shown = tick.max(self.shown);
        Some(self.shown)
    }
}

/// Splits `tick`, counted from Start, into the lap playing and the tick
/// within it, as the engine picks laps: in turn, one a pass.
pub fn locate(tick: f64, len: u32, laps: usize) -> Playhead {
    let len = f64::from(len);
    let pass = (tick / len).floor();
    Playhead {
        lap: pass as usize % laps,
        tick: tick - pass * len,
    }
}
