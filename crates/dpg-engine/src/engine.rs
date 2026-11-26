//! The audio thread's side: plays the pattern's laps on the clock through the
//! kit, mixes the voices and publishes where playback is. Patterns and
//! kits arrive whole through queues, and the ones replaced go back the
//! same way to be freed off the audio thread.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU16, AtomicU32, AtomicU64, Ordering};

use dpg_hal::{Block, Render};
use dpg_samples::Kit;
use drum_patterns::{HitKind, Instrument, Pattern, TimeSignature};
use rtrb::{Consumer, Producer, RingBuffer};

use crate::clock::Clock;
use crate::metronome::{Click, Clicks, Subdivision, click_at};
use crate::mix::{CHANNELS, Channel, MixControls, gains};
use crate::remote::Remote;
use crate::rules::{BarSkip, CrashRule};
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
/// The clicks' ids follow every kit's.
const FIRST_CLICK: usize = KIT_SLOTS * IDS_PER_KIT;
const CLICKS: [Click; 3] = [Click::Bar, Click::Beat, Click::Between];
/// How hard the crash rule hits - by ear in 4g.
const CRASH_KIND: HitKind = HitKind::Accent;

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
    pub(crate) metronome: AtomicBool,
    /// A [`Subdivision`]'s index.
    pub(crate) subdivision: AtomicU8,
    pub(crate) bar_skip: AtomicU16,
    pub(crate) crash: AtomicU16,
}

/// What the audio thread hands back to be freed.
#[derive(Debug)]
#[expect(dead_code, reason = "held only to be dropped on the UI thread")]
pub(crate) enum Garbage {
    Laps(Vec<Pattern>),
    Kit(Kit),
}

/// Plays a groove into the blocks an output asks for. Lives on the audio
/// thread; the UI drives it through its [`Remote`].
#[derive(Debug)]
pub struct Engine {
    rate: u32,
    shared: Arc<Shared>,
    new_laps: Consumer<Vec<Pattern>>,
    new_kits: Consumer<Kit>,
    garbage: Producer<Garbage>,
    /// The pattern on each time through, in turn: one, or up to three for
    /// a dotted hi-hat. Never empty, all the same length.
    laps: Vec<Pattern>,
    /// `kits[current]` plays new hits; the others ring out.
    kits: [Option<Kit>; KIT_SLOTS],
    current: usize,
    clock: Clock,
    playing: bool,
    starts: u32,
    voices: Voices,
    clicks: Clicks,
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
            metronome: AtomicBool::new(false),
            subdivision: AtomicU8::new(Subdivision::default().index()),
            bar_skip: AtomicU16::new(BarSkip::pack(None)),
            crash: AtomicU16::new(CrashRule::default().pack()),
        });
        let (pattern_tx, pattern_rx) = RingBuffer::new(2);
        let (kit_tx, kit_rx) = RingBuffer::new(2);
        let (garbage_tx, garbage_rx) = RingBuffer::new(KIT_SLOTS + 4);
        let engine = Self {
            rate,
            shared: shared.clone(),
            new_laps: pattern_rx,
            new_kits: kit_rx,
            garbage: garbage_tx,
            laps: vec![Pattern::new(time, 1).expect("one bar is in range")],
            kits: [Some(Kit::new(rate)), None, None, None],
            current: 0,
            clock: Clock::new(rate, time, DEFAULT_BPM),
            playing: false,
            starts: 0,
            voices: Voices::new(rate),
            clicks: Clicks::new(rate),
            gains: None,
        };
        let remote = Remote::new(rate, shared, pattern_tx, kit_tx, garbage_rx);
        (engine, remote)
    }

    /// Swaps in the newest of what the UI sent; anything older goes
    /// straight back. New laps play at once from the same tick; laps of
    /// another time signature start their bar over.
    fn receive(&mut self) {
        while self.garbage.slots() > 0
            && let Ok(laps) = self.new_laps.pop()
        {
            let old = std::mem::replace(&mut self.laps, laps);
            self.throw(Garbage::Laps(old));
        }
        let time = self.laps[0].time();
        if time != self.clock.time() {
            self.clock = Clock::new(self.rate, time, self.clock.bpm());
        }
        while let Some(free) = self.kits.iter().position(Option::is_none)
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
            self.clock = Clock::new(self.rate, self.laps[0].time(), bpm);
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

    /// Starts a voice for every hit and click on a tick in the next
    /// `frames`. The laps take turns, and bar skip and the crash go by
    /// the bar of playback.
    fn schedule(&mut self, frames: usize) {
        let shared = &self.shared;
        let metronome = shared
            .metronome
            .load(Ordering::Relaxed)
            .then(|| Subdivision::from_index(shared.subdivision.load(Ordering::Relaxed)));
        let skip = BarSkip::unpack(shared.bar_skip.load(Ordering::Relaxed));
        let crash = CrashRule::unpack(shared.crash.load(Ordering::Relaxed));
        let Self {
            clock,
            laps,
            kits,
            current,
            voices,
            clicks,
            ..
        } = self;
        let time = clock.time();
        let per_bar = u64::from(time.ticks_per_bar());
        let len = u64::from(laps[0].len_ticks());
        let kit = kits[*current].as_ref();
        let drum = |instrument: Instrument, kind: HitKind, offset: usize| {
            let sound = kit?.sound(instrument)?;
            Some(Trigger {
                sample: sample_id(*current, instrument, kind),
                channel: instrument.into(),
                gain: sound.hit(kind).1,
                offset,
            })
        };
        clock.advance(frames, |t| {
            let bar = t.tick / per_bar;
            if BarSkip::allows(skip, bar) {
                let lap = &laps[(t.tick / len % laps.len() as u64) as usize];
                let at = (t.tick % len) as u32;
                let hits = lap.hits();
                let on_tick = &hits[hits.partition_point(|h| h.tick < at)..];
                let on_tick = on_tick.iter().take_while(|h| h.tick == at);
                let mut crashed = false;
                for h in on_tick {
                    crashed |= h.instrument == Instrument::Crash;
                    if let Some(trigger) = drum(h.instrument, h.kind, t.offset) {
                        voices.trigger(trigger);
                    }
                }
                // The crash rule adds one unless the pattern has its own.
                if !crashed
                    && t.tick % per_bar == 0
                    && crash.plays(bar)
                    && let Some(trigger) = drum(Instrument::Crash, CRASH_KIND, t.offset)
                {
                    voices.trigger(trigger);
                }
            }
            if let Some(subdivision) = metronome
                && let Some(click) = click_at(t.tick, time, subdivision)
            {
                voices.trigger(Trigger {
                    sample: click_id(click),
                    channel: Channel::Metronome,
                    gain: clicks.sound(click).1,
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
        let (kits, clicks) = (&self.kits, &self.clicks);
        self.voices
            .mix(block.out, block.channels, &from, &to, |id| {
                lookup(kits, clicks, id)
            });
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

fn click_id(click: Click) -> SampleId {
    let index = CLICKS
        .iter()
        .position(|&c| c == click)
        .expect("CLICKS lists every click");
    SampleId((FIRST_CLICK + index) as u32)
}

/// The kit slot `id` plays from; past the last for a click.
fn slot_of(id: SampleId) -> usize {
    id.0 as usize / IDS_PER_KIT
}

/// The samples behind `id`; empty if its kit or sound is gone.
fn lookup<'a>(kits: &'a [Option<Kit>; KIT_SLOTS], clicks: &'a Clicks, id: SampleId) -> &'a [f32] {
    if let Some(&click) = CLICKS.get((id.0 as usize).wrapping_sub(FIRST_CLICK)) {
        return clicks.sound(click).0;
    }
    let n = id.0 as usize % IDS_PER_KIT;
    let (instrument, kind) = (Instrument::ALL[n / KINDS.len()], KINDS[n % KINDS.len()]);
    kits.get(slot_of(id))
        .and_then(Option::as_ref)
        .and_then(|kit| kit.sound(instrument))
        .map_or(&[], |sound| sound.hit(kind).0)
}
