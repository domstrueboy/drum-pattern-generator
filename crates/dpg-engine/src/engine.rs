//! The audio thread's side: plays the pattern on the clock through the
//! kit, mixes the voices and publishes where playback is. Patterns and
//! kits arrive whole through queues, and the ones replaced go back the
//! same way to be freed off the audio thread.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

use dpg_hal::{Block, Render};
use dpg_samples::Kit;
use drum_patterns::{HitKind, Instrument, Pattern, TimeSignature};
use rtrb::{Consumer, Producer, RingBuffer};

use crate::clock::Clock;
use crate::mix::{CHANNELS, Channel, MixControls, gains};
use crate::remote::Remote;
use crate::voices::{SampleId, Trigger, Voices};

#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;

/// The tempo until the UI sets one.
pub const DEFAULT_BPM: f64 = 120.0;

/// Kits held at once: the one playing, and replaced ones still ringing.
/// A new kit waits in its queue while all are taken.
const KIT_SLOTS: usize = 4;
const KINDS: [HitKind; 3] = [HitKind::Accent, HitKind::Normal, HitKind::Ghost];
const IDS_PER_KIT: usize = Instrument::ALL.len() * KINDS.len();

/// What the UI and the audio thread both see. Atomics only.
#[derive(Debug)]
pub(crate) struct Shared {
    pub(crate) play: AtomicBool,
    /// Counts presses of Start, so Stop then Start between two blocks
    /// still starts over.
    pub(crate) starts: AtomicU32,
    pub(crate) bpm: AtomicU64,
    /// The tick being heard, as `f64` bits; NaN when stopped.
    pub(crate) position: AtomicU64,
    pub(crate) mix: MixControls,
}

/// What the audio thread hands back to be freed.
#[derive(Debug)]
#[expect(dead_code, reason = "held only to be dropped on the UI thread")]
pub(crate) enum Garbage {
    Pattern(Pattern),
    Kit(Kit),
}

/// Plays a groove into the blocks an output asks for. Lives on the audio
/// thread; the UI drives it through its [`Remote`].
#[derive(Debug)]
pub struct Engine {
    rate: u32,
    shared: Arc<Shared>,
    patterns: Consumer<Pattern>,
    new_kits: Consumer<Kit>,
    garbage: Producer<Garbage>,
    pattern: Pattern,
    /// `kits[current]` plays new hits; the others ring out.
    kits: [Option<Kit>; KIT_SLOTS],
    current: usize,
    clock: Clock,
    playing: bool,
    starts: u32,
    voices: Voices,
    /// The gains the last block ended on; none before the first.
    gains: Option<[f32; CHANNELS]>,
}

impl Engine {
    /// A stopped engine for an output at `rate`, with an empty kit and an
    /// empty bar of 4/4, and the remote that drives it.
    pub fn new(rate: u32) -> (Self, Remote) {
        let time = TimeSignature::FOUR_FOUR;
        let shared = Arc::new(Shared {
            play: AtomicBool::new(false),
            starts: AtomicU32::new(0),
            bpm: AtomicU64::new(DEFAULT_BPM.to_bits()),
            position: AtomicU64::new(f64::NAN.to_bits()),
            mix: MixControls::default(),
        });
        let (pattern_tx, pattern_rx) = RingBuffer::new(2);
        let (kit_tx, kit_rx) = RingBuffer::new(2);
        let (garbage_tx, garbage_rx) = RingBuffer::new(KIT_SLOTS + 4);
        let engine = Self {
            rate,
            shared: shared.clone(),
            patterns: pattern_rx,
            new_kits: kit_rx,
            garbage: garbage_tx,
            pattern: Pattern::new(time, 1).expect("one bar is in range"),
            kits: [Some(Kit::new(rate)), None, None, None],
            current: 0,
            clock: Clock::new(rate, time, DEFAULT_BPM),
            playing: false,
            starts: 0,
            voices: Voices::new(rate),
            gains: None,
        };
        let remote = Remote::new(rate, shared, pattern_tx, kit_tx, garbage_rx);
        (engine, remote)
    }

    /// Swaps in what the UI sent. A new pattern plays at once from the
    /// same tick; one of another time signature starts its bar over.
    fn receive(&mut self) {
        if self.garbage.slots() > 0
            && let Ok(pattern) = self.patterns.pop()
        {
            let old = std::mem::replace(&mut self.pattern, pattern);
            self.throw(Garbage::Pattern(old));
            if self.pattern.time() != self.clock.time() {
                self.clock = Clock::new(self.rate, self.pattern.time(), self.clock.bpm());
            }
        }
        if let Some(free) = self.kits.iter().position(Option::is_none)
            && let Ok(kit) = self.new_kits.pop()
        {
            self.kits[free] = Some(kit);
            self.current = free;
        }
    }

    /// Follows Start, Stop and the tempo.
    fn transport(&mut self) {
        let starts = self.shared.starts.load(Ordering::Acquire);
        let play = self.shared.play.load(Ordering::Relaxed);
        let bpm = f64::from_bits(self.shared.bpm.load(Ordering::Relaxed));
        if play && starts != self.starts {
            self.voices.release_all();
            self.clock = Clock::new(self.rate, self.pattern.time(), bpm);
            self.playing = true;
        } else if !play && self.playing {
            self.voices.release_all();
            self.playing = false;
        }
        self.starts = starts;
        if bpm != self.clock.bpm() {
            self.clock.set_bpm(bpm);
        }
    }

    /// Publishes the tick being heard now: the block's start, less the
    /// output's latency.
    fn publish(&self, latency: f64) {
        let heard = if self.playing {
            let frame = self.clock.frame() as f64 - latency * f64::from(self.rate);
            self.clock.tick_at(frame).max(0.0)
        } else {
            f64::NAN
        };
        self.shared
            .position
            .store(heard.to_bits(), Ordering::Relaxed);
    }

    /// Starts a voice for every hit on a tick in the next `frames`.
    fn schedule(&mut self, frames: usize) {
        let Self {
            clock,
            pattern,
            kits,
            current,
            voices,
            ..
        } = self;
        let len = u64::from(pattern.len_ticks());
        let hits = pattern.hits();
        let kit = kits[*current].as_ref();
        clock.advance(frames, |t| {
            let at = (t.tick % len) as u32;
            let first = hits.partition_point(|h| h.tick < at);
            for hit in hits[first..].iter().take_while(|h| h.tick == at) {
                let Some(sound) = kit.and_then(|k| k.sound(hit.instrument)) else {
                    continue;
                };
                voices.trigger(Trigger {
                    sample: sample_id(*current, hit.instrument, hit.kind),
                    channel: hit.instrument.into(),
                    gain: sound.hit(hit.kind).1,
                    offset: t.offset,
                });
            }
        });
    }

    /// Sends back each replaced kit no voice plays any more.
    fn retire(&mut self) {
        for slot in 0..KIT_SLOTS {
            if slot == self.current
                || self.kits[slot].is_none()
                || self.garbage.slots() == 0
                || self.voices.any(|id| slot_of(id) == slot)
            {
                continue;
            }
            if let Some(kit) = self.kits[slot].take() {
                self.throw(Garbage::Kit(kit));
            }
        }
    }

    /// Hands `garbage` back. Only called with room in the queue, so it is
    /// never dropped here.
    fn throw(&mut self, garbage: Garbage) {
        let sent = self.garbage.push(garbage);
        debug_assert!(sent.is_ok(), "checked for room first");
    }
}

impl Render for Engine {
    fn render(&mut self, block: Block<'_>) {
        let frames = block.out.len() / block.channels;
        self.receive();
        self.transport();
        self.publish(block.latency);
        if self.playing {
            self.schedule(frames);
        }
        let to = gains(&self.shared.mix.snapshot());
        let from = self.gains.unwrap_or(to);
        self.gains = Some(to);
        let kits = &self.kits;
        self.voices
            .mix(block.out, block.channels, &from, &to, |id| lookup(kits, id));
        self.retire();
    }
}

/// The id of what a hit of `kind` on `instrument` plays from the kit in
/// `slot`.
fn sample_id(slot: usize, instrument: Instrument, kind: HitKind) -> SampleId {
    let instrument = Channel::from(instrument).index();
    let kind = KINDS
        .iter()
        .position(|&k| k == kind)
        .expect("KINDS lists every kind");
    SampleId((slot * IDS_PER_KIT + instrument * KINDS.len() + kind) as u32)
}

fn slot_of(id: SampleId) -> usize {
    id.0 as usize / IDS_PER_KIT
}

/// The samples behind `id`; empty if its kit or sound is gone.
fn lookup(kits: &[Option<Kit>; KIT_SLOTS], id: SampleId) -> &[f32] {
    let n = id.0 as usize % IDS_PER_KIT;
    let (instrument, kind) = (Instrument::ALL[n / KINDS.len()], KINDS[n % KINDS.len()]);
    kits.get(slot_of(id))
        .and_then(Option::as_ref)
        .and_then(|kit| kit.sound(instrument))
        .map_or(&[], |sound| sound.hit(kind).0)
}
