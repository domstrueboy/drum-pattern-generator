//! The UI's side of the engine: Start, Stop, tempo and mix as atomics; a
//! pattern or kit sent whole; the position read back. Never blocks.

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use dpg_samples::Kit;
use drum_patterns::Pattern;
use rtrb::{Consumer, Producer, PushError};

use crate::clicks::ClickSound;
use crate::clock::{BPM_MAX, BPM_MIN};
use crate::engine::{Garbage, Shared};
use crate::metronome::Subdivision;
use crate::mix::MixControls;
use crate::rules::{BarSkip, CrashRule};
use crate::tap::TapTempo;

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
    laps: Producer<Vec<Pattern>>,
    kits: Producer<Kit>,
    garbage: Consumer<Garbage>,
    waiting_laps: Option<Vec<Pattern>>,
    kit: Option<Kit>,
    tap: TapTempo,
}

impl Remote {
    pub(crate) fn new(
        rate: u32,
        shared: Arc<Shared>,
        laps: Producer<Vec<Pattern>>,
        kits: Producer<Kit>,
        garbage: Consumer<Garbage>,
    ) -> Self {
        Self {
            rate,
            shared,
            laps,
            kits,
            garbage,
            waiting_laps: None,
            kit: None,
            tap: TapTempo::new(),
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

    /// A tap of Tap tempo at `seconds` on any steady clock. From the
    /// second tap on, sets the tempo and returns it.
    pub fn tap(&mut self, seconds: f64) -> Option<f64> {
        let bpm = self.tap.tap(seconds)?;
        self.set_bpm(bpm);
        Some(bpm)
    }

    /// Turns the metronome on or off. It clicks while playing.
    pub fn set_metronome(&self, on: bool) {
        self.shared.metronome.store(on, Ordering::Relaxed);
    }

    /// Whether the metronome is on.
    pub fn metronome(&self) -> bool {
        self.shared.metronome.load(Ordering::Relaxed)
    }

    /// How many clicks a beat.
    pub fn set_subdivision(&self, subdivision: Subdivision) {
        self.shared
            .subdivision
            .store(subdivision.index(), Ordering::Relaxed);
    }

    /// The clicks a beat.
    pub fn subdivision(&self) -> Subdivision {
        Subdivision::from_index(self.shared.subdivision.load(Ordering::Relaxed))
    }

    /// What the metronome sounds like; a click already ringing finishes
    /// in the old sound.
    pub fn set_click_sound(&self, sound: ClickSound) {
        self.shared
            .click_sound
            .store(sound.index(), Ordering::Relaxed);
    }

    /// The metronome's sound.
    pub fn click_sound(&self) -> ClickSound {
        ClickSound::from_index(self.shared.click_sound.load(Ordering::Relaxed))
    }

    /// The mixer's faders, solo and mute.
    pub fn mix(&self) -> &MixControls {
        &self.shared.mix
    }

    /// Plays `pattern` from the next block on, at the same place in the
    /// bar.
    pub fn set_pattern(&mut self, pattern: Pattern) {
        self.set_laps(vec![pattern]);
    }

    /// Plays `laps` in turn, one each time through, from the next block
    /// on: what [`drum_patterns::Groove::build_laps`] makes for a dotted
    /// hi-hat. Lap `n` of playback is `laps[n % laps.len()]`.
    ///
    /// # Panics
    ///
    /// If `laps` is empty or they differ in length or time signature.
    pub fn set_laps(&mut self, laps: Vec<Pattern>) {
        let first = laps.first().expect("at least one lap");
        assert!(
            laps.iter()
                .all(|l| l.len_ticks() == first.len_ticks() && l.time() == first.time()),
            "every lap the same length"
        );
        self.waiting_laps = Some(laps);
        self.poll();
    }

    /// Turns bar skip on with a round of bars, or off with `None`. Takes
    /// effect from the next bar - a round counts from Start.
    pub fn set_bar_skip(&self, skip: Option<BarSkip>) {
        self.shared
            .bar_skip
            .store(BarSkip::pack(skip), Ordering::Relaxed);
    }

    /// Bar skip's round, if on.
    pub fn bar_skip(&self) -> Option<BarSkip> {
        BarSkip::unpack(self.shared.bar_skip.load(Ordering::Relaxed))
    }

    /// Where the crash plays.
    pub fn set_crash(&self, crash: CrashRule) {
        self.shared.crash.store(crash.pack(), Ordering::Relaxed);
    }

    /// Where the crash plays.
    pub fn crash(&self) -> CrashRule {
        CrashRule::unpack(self.shared.crash.load(Ordering::Relaxed))
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
    /// Returns how many sets of laps and kits it freed.
    pub fn poll(&mut self) -> usize {
        let mut freed = 0;
        while self.garbage.pop().is_ok() {
            freed += 1;
        }
        send(&mut self.laps, &mut self.waiting_laps);
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
