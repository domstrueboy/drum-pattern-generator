//! Musical time in frames: which tick of the groove falls on which frame,
//! at a tempo that may change while playing.

use drum_patterns::TimeSignature;

#[cfg(test)]
#[path = "clock_tests.rs"]
mod tests;

/// The slowest tempo, in beats a minute.
pub const BPM_MIN: f64 = 30.0;
/// The fastest tempo, in beats a minute.
pub const BPM_MAX: f64 = 300.0;

/// A tick that falls in the block being played.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tick {
    /// Frames from the start of the block.
    pub offset: usize,
    /// Ticks since playing started - the groove's position is this modulo
    /// its length, the bar of playback this over the ticks in a bar.
    pub tick: u64,
}

/// Where the ticks fall, from the start of playing.
///
/// Each tick's frame is rounded from its exact position, never from the
/// tick before, so the grid can't wander: at 92 BPM a tick is 2,396.74
/// frames, and rounding that once would drift a frame every few ticks.
#[derive(Debug, Clone)]
pub struct Clock {
    rate: f64,
    time: TimeSignature,
    bpm: f64,
    frames_per_tick: f64,
    /// A frame and the exact (fractional) tick on it; the tempo holds from
    /// here on. Moves at each tempo change.
    anchor_frame: u64,
    anchor_tick: f64,
    /// The next frame to play.
    frame: u64,
    /// The next tick not yet played.
    next_tick: u64,
}

impl Clock {
    /// A clock at frame 0, tick 0. `bpm` counts the time signature's beats
    /// (quarters in 4/4, eighths in 6/8) and is clamped to
    /// [`BPM_MIN`]..=[`BPM_MAX`].
    pub fn new(rate: u32, time: TimeSignature, bpm: f64) -> Self {
        let mut clock = Self {
            rate: f64::from(rate),
            time,
            bpm: 0.0,
            frames_per_tick: 0.0,
            anchor_frame: 0,
            anchor_tick: 0.0,
            frame: 0,
            next_tick: 0,
        };
        clock.set_bpm(bpm);
        clock
    }

    /// The tempo, in beats a minute.
    pub fn bpm(&self) -> f64 {
        self.bpm
    }

    /// The time signature.
    pub fn time(&self) -> TimeSignature {
        self.time
    }

    /// Changes the tempo from the next frame on. The beat carries on from
    /// where it is - nothing skips or repeats.
    pub fn set_bpm(&mut self, bpm: f64) {
        let bpm = bpm.clamp(BPM_MIN, BPM_MAX);
        self.anchor_tick = self.tick_at(self.frame as f64);
        self.anchor_frame = self.frame;
        self.bpm = bpm;
        let ticks_per_second = bpm / 60.0 * f64::from(self.time.ticks_per_beat());
        self.frames_per_tick = self.rate / ticks_per_second;
    }

    /// Plays the next `frames` frames: calls `on_tick` for each tick in
    /// them, in order. Allocates nothing.
    pub fn advance(&mut self, frames: usize, mut on_tick: impl FnMut(Tick)) {
        let end = self.frame + frames as u64;
        loop {
            let at = self.frame_of(self.next_tick);
            if at >= end as f64 - 0.5 {
                break;
            }
            // A tick just before a tempo change can round onto the frame
            // before the block; it still plays, at the block's start.
            let offset = (at.round() - self.frame as f64).max(0.0) as usize;
            on_tick(Tick {
                offset,
                tick: self.next_tick,
            });
            self.next_tick += 1;
        }
        self.frame = end;
    }

    /// The next frame to play.
    pub fn frame(&self) -> u64 {
        self.frame
    }

    /// The exact position, in ticks, at `frame` - fractional, and before
    /// the next frame if a latency is taken off. For the UI's playhead.
    pub fn tick_at(&self, frame: f64) -> f64 {
        if self.frames_per_tick == 0.0 {
            return 0.0;
        }
        self.anchor_tick + (frame - self.anchor_frame as f64) / self.frames_per_tick
    }

    /// The bar of playback `tick` is in, from 0.
    pub fn bar_of(&self, tick: u64) -> u64 {
        tick / u64::from(self.time.ticks_per_bar())
    }

    /// The exact frame `tick` falls on, at the current tempo.
    fn frame_of(&self, tick: u64) -> f64 {
        self.anchor_frame as f64 + (tick as f64 - self.anchor_tick) * self.frames_per_tick
    }
}
