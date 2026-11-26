//! The UI's side of the engine: Start, Stop, tempo and mix as atomics; a
//! pattern or kit sent whole; the position read back. Never blocks.

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use dpg_samples::Kit;
use drum_patterns::Pattern;
use rtrb::{Consumer, Producer, PushError};

use crate::clock::{BPM_MAX, BPM_MIN};
use crate::engine::{Garbage, Shared};
use crate::mix::MixControls;

#[cfg(test)]
#[path = "remote_tests.rs"]
mod tests;

/// A kit made for another sample rate than the output's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WrongRate {
    /// The kit's rate.
    pub kit: u32,
    /// The output's rate.
    pub output: u32,
}

impl fmt::Display for WrongRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the kit is at {} Hz, the output at {} Hz",
            self.kit, self.output
        )
    }
}

impl std::error::Error for WrongRate {}

/// Drives an [`Engine`](crate::Engine) from the UI thread.
///
/// A pattern or kit the audio thread hasn't taken yet waits here, and a
/// newer one replaces it. Call [`Remote::poll`] now and then - with each
/// position read is enough - to send what waits and free what came back.
#[derive(Debug)]
pub struct Remote {
    rate: u32,
    shared: Arc<Shared>,
    patterns: Producer<Pattern>,
    kits: Producer<Kit>,
    garbage: Consumer<Garbage>,
    pattern: Option<Pattern>,
    kit: Option<Kit>,
}

impl Remote {
    pub(crate) fn new(
        rate: u32,
        shared: Arc<Shared>,
        patterns: Producer<Pattern>,
        kits: Producer<Kit>,
        garbage: Consumer<Garbage>,
    ) -> Self {
        Self {
            rate,
            shared,
            patterns,
            kits,
            garbage,
            pattern: None,
            kit: None,
        }
    }

    /// Start: plays the pattern from its first tick, starting over if it
    /// was playing.
    pub fn play(&self) {
        self.shared.play.store(true, Ordering::Relaxed);
        self.shared.starts.fetch_add(1, Ordering::Release);
    }

    /// Stop: what's sounding fades out in a few milliseconds.
    pub fn stop(&self) {
        self.shared.play.store(false, Ordering::Relaxed);
    }

    /// Whether Start was pressed last, rather than Stop.
    pub fn is_playing(&self) -> bool {
        self.shared.play.load(Ordering::Relaxed)
    }

    /// Sets the tempo, clamped to the engine's range. Takes effect from the
    /// next block, carrying the beat on.
    pub fn set_bpm(&self, bpm: f64) {
        let bpm = bpm.clamp(BPM_MIN, BPM_MAX);
        self.shared.bpm.store(bpm.to_bits(), Ordering::Relaxed);
    }

    /// The tempo, in beats a minute.
    pub fn bpm(&self) -> f64 {
        f64::from_bits(self.shared.bpm.load(Ordering::Relaxed))
    }

    /// The mixer's faders, solo and mute.
    pub fn mix(&self) -> &MixControls {
        &self.shared.mix
    }

    /// Plays `pattern` from the next block on, at the same place in the
    /// bar.
    pub fn set_pattern(&mut self, pattern: Pattern) {
        self.pattern = Some(pattern);
        self.poll();
    }

    /// Plays new hits on `kit` from the next block on; hits still ringing
    /// finish on the old one. The kit must be at the output's rate.
    pub fn set_kit(&mut self, kit: Kit) -> Result<(), WrongRate> {
        if kit.rate() != self.rate {
            return Err(WrongRate {
                kit: kit.rate(),
                output: self.rate,
            });
        }
        self.kit = Some(kit);
        self.poll();
        Ok(())
    }

    /// The tick being heard, counted from Start (the pattern's tick is this
    /// modulo its length); `None` when stopped. Behind the audio by up to a
    /// block.
    pub fn position(&self) -> Option<f64> {
        let tick = f64::from_bits(self.shared.position.load(Ordering::Relaxed));
        (!tick.is_nan()).then_some(tick)
    }

    /// Frees what the audio thread handed back and sends what waits.
    /// Returns how many patterns and kits it freed.
    pub fn poll(&mut self) -> usize {
        let mut freed = 0;
        while self.garbage.pop().is_ok() {
            freed += 1;
        }
        send(&mut self.patterns, &mut self.pattern);
        send(&mut self.kits, &mut self.kit);
        freed
    }
}

fn send<T>(queue: &mut Producer<T>, waiting: &mut Option<T>) {
    if let Some(value) = waiting.take()
        && let Err(PushError::Full(value)) = queue.push(value)
    {
        *waiting = Some(value);
    }
}
