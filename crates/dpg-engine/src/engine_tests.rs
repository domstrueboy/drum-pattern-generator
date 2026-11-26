use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use dpg_hal::MockOutput;
use dpg_samples::Sound;
use drum_patterns::Hit;
use drum_patterns::Instrument::{Crash, HiHat, Kick, Snare};

use super::*;
use crate::voices::FADE_MS;

const RATE: u32 = 48_000;
/// At 120 BPM a tick is 2,000 frames and a bar of 4/4 96,000.
const TICK: usize = 2_000;
const BAR: usize = 48 * TICK;
const BLOCK: usize = 256;

/// Every instrument plays `len` frames at its own level, so the output
/// shows which one sounded: crash 0.1, hi-hat 0.2, snare 0.3, kick 0.4,
/// times `scale`.
fn kit(len: usize, scale: f32) -> Kit {
    let mut kit = Kit::new(RATE);
    for (i, instrument) in Instrument::ALL.into_iter().enumerate() {
        let level = 0.1 * (i + 1) as f32 * scale;
        kit.set(instrument, Sound::new(vec![level; len], None));
    }
    kit
}

/// Accents, which play at full gain, over `bars` of 4/4.
fn pattern(bars: u8, hits: &[(u32, Instrument)]) -> Pattern {
    let mut pattern = Pattern::new(TimeSignature::FOUR_FOUR, bars).unwrap();
    for &(tick, instrument) in hits {
        pattern
            .set(Hit::new(tick, instrument, HitKind::Accent))
            .unwrap();
    }
    pattern
}

/// An engine with the Drums fader up, so an accent sounds at its sample's
/// level.
fn ready(kit: Kit, pattern: Pattern) -> (Engine, Remote) {
    let (engine, mut remote) = Engine::new(RATE);
    remote.mix().set_drums(1.0);
    remote.set_kit(kit).unwrap();
    remote.set_pattern(pattern);
    (engine, remote)
}

/// The next `frames` mono frames, a block at a time.
fn render(engine: &mut Engine, frames: usize) -> Vec<f32> {
    render_late(engine, frames, 0.0)
}

fn render_late(engine: &mut Engine, frames: usize, latency: f64) -> Vec<f32> {
    let mut out = vec![0.0; frames];
    for block in out.chunks_mut(BLOCK) {
        engine.render(Block {
            out: block,
            channels: 1,
            latency,
        });
    }
    out
}

/// Where a sound starts after silence, and at what level in thousandths.
fn onsets(out: &[f32]) -> Vec<(usize, i32)> {
    (0..out.len())
        .filter(|&i| out[i] != 0.0 && (i == 0 || out[i - 1] == 0.0))
        .map(|i| (i, milli(out[i])))
        .collect()
}

fn milli(sample: f32) -> i32 {
    (sample * 1000.0).round() as i32
}

#[test]
fn silent_until_start() {
    let (mut engine, _remote) = ready(kit(100, 1.0), pattern(1, &[(0, Kick)]));
    assert!(render(&mut engine, BAR).iter().all(|&s| s == 0.0));
}

#[test]
fn each_hit_sounds_on_its_frame_bar_after_bar() {
    let groove = pattern(1, &[(0, Kick), (12, HiHat), (24, Snare), (36, HiHat)]);
    let (mut engine, remote) = ready(kit(100, 1.0), groove);
    remote.play();
    let bar = [
        (0, 400),
        (12 * TICK, 200),
        (24 * TICK, 300),
        (36 * TICK, 200),
    ];
    let expected: Vec<_> = (0..2)
        .flat_map(|n| bar.map(|(at, level)| (n * BAR + at, level)))
        .collect();
    assert_eq!(onsets(&render(&mut engine, 2 * BAR)), expected);
}

#[test]
fn stop_fades_out_and_start_plays_from_the_top() {
    let (mut engine, remote) = ready(kit(10 * BAR, 1.0), pattern(1, &[(0, Kick)]));
    remote.play();
    render(&mut engine, 40 * BLOCK);
    remote.stop();
    let out = render(&mut engine, 4 * BLOCK);
    let fade = (RATE * FADE_MS / 1000) as usize;
    assert_eq!(milli(out[0]), 400);
    assert!(out[fade - 1] > 0.0);
    assert!(out[fade..].iter().all(|&s| s == 0.0), "silent once faded");

    remote.play();
    let out = render(&mut engine, BLOCK);
    assert_eq!(milli(out[0]), 400, "the first kick again");
}

#[test]
fn a_new_pattern_plays_at_once_from_the_same_place() {
    let (mut engine, mut remote) = ready(kit(100, 1.0), pattern(1, &[(0, Kick)]));
    remote.play();
    let mut out = render(&mut engine, 40 * BLOCK);
    // Past tick 5: the new hit on tick 2 waits for the next bar.
    remote.set_pattern(pattern(1, &[(0, Kick), (2, HiHat), (24, Snare)]));
    out.extend(render(&mut engine, 2 * BAR - out.len()));
    let expected = vec![
        (0, 400),
        (24 * TICK, 300),
        (BAR, 400),
        (BAR + 2 * TICK, 200),
        (BAR + 24 * TICK, 300),
    ];
    assert_eq!(onsets(&out), expected);
}

#[test]
fn the_newest_pattern_wins() {
    let (mut engine, mut remote) = ready(kit(100, 1.0), pattern(1, &[]));
    for _ in 0..3 {
        remote.set_pattern(pattern(1, &[(0, Kick)]));
    }
    remote.set_pattern(pattern(1, &[(0, Snare)]));
    for _ in 0..4 {
        render(&mut engine, BLOCK);
        remote.poll();
    }
    remote.play();
    assert_eq!(onsets(&render(&mut engine, BLOCK)), [(0, 300)]);
}

#[test]
fn a_replaced_kit_rings_out_then_comes_back() {
    let ring = 30_000;
    let (mut engine, mut remote) = ready(kit(ring, 1.0), pattern(1, &[(0, Kick)]));
    remote.play();
    let mut out = render(&mut engine, 4 * BLOCK);
    assert_eq!(
        remote.poll(),
        2,
        "the empty pattern and kit it started with"
    );

    remote.set_kit(kit(100, 2.0)).unwrap();
    out.extend(render(&mut engine, 70 * BLOCK));
    assert_eq!(remote.poll(), 0, "the old kit still rings");

    out.extend(render(&mut engine, BAR + BLOCK - out.len()));
    assert_eq!(remote.poll(), 1, "the old kit, once silent");
    assert_eq!(milli(out[ring - 1]), 400, "rings to its end");
    assert_eq!(out[ring], 0.0);
    assert_eq!(onsets(&out), [(0, 400), (BAR, 800)]);
}

#[test]
fn a_tempo_change_carries_the_beat_on() {
    let beats = pattern(1, &[(0, Kick), (12, Kick), (24, Kick), (36, Kick)]);
    let (mut engine, remote) = ready(kit(100, 1.0), beats);
    remote.play();
    let mut out = render(&mut engine, 94 * BLOCK);
    remote.set_bpm(60.0);
    out.extend(render(&mut engine, 80_000 - out.len()));
    // 12.032 ticks in at 120 BPM; 11.968 more at 4,000 frames a tick.
    let at: Vec<_> = onsets(&out).into_iter().map(|(at, _)| at).collect();
    assert_eq!(at, [0, 12 * TICK, 94 * BLOCK + 47_872]);
}

#[test]
fn muting_the_kick_leaves_the_snare() {
    let (mut engine, remote) = ready(kit(100, 1.0), pattern(1, &[(0, Kick), (24, Snare)]));
    remote.mix().set_mute(Channel::Kick, true);
    remote.play();
    assert_eq!(onsets(&render(&mut engine, BAR)), [(24 * TICK, 300)]);
}

#[test]
fn the_position_is_the_tick_being_heard() {
    let (mut engine, remote) = ready(kit(100, 1.0), pattern(1, &[(0, Kick)]));
    let latency = 0.01;
    render_late(&mut engine, BLOCK, latency);
    assert_eq!(remote.position(), None, "stopped");

    remote.play();
    render_late(&mut engine, 40 * BLOCK, latency);
    // The last block starts on frame 9,984; 480 frames are still on their
    // way out.
    let heard = (39 * BLOCK - 480) as f64 / TICK as f64;
    assert!((remote.position().unwrap() - heard).abs() < 1e-9);

    remote.stop();
    render_late(&mut engine, BLOCK, latency);
    assert_eq!(remote.position(), None);
}

#[test]
fn plays_on_every_channel_of_an_output() {
    let (engine, remote) = ready(kit(100, 1.0), pattern(1, &[(0, Crash)]));
    let mut output = MockOutput::new(RATE, 2, engine);
    remote.play();
    let out = output.pull(BLOCK);
    assert_eq!((milli(out[0]), milli(out[1])), (100, 100));
}

#[test]
fn the_audio_thread_never_allocates_or_frees() {
    // A 16th on every instrument at 300 BPM with long sounds: the pool
    // fills and steals.
    let busy: Vec<_> = (0..48)
        .step_by(3)
        .flat_map(|t| Instrument::ALL.map(|i| (t, i)))
        .collect();
    let (mut engine, mut remote) = ready(kit(BAR, 0.1), pattern(1, &busy));
    remote.set_bpm(300.0);
    let mut out = vec![0.0; BLOCK];
    let mut play = |engine: &mut Engine, blocks: usize| {
        allocations(|| {
            for _ in 0..blocks {
                out.fill(0.0);
                engine.render(Block {
                    out: &mut out,
                    channels: 1,
                    latency: 0.0,
                });
            }
        })
    };

    remote.play();
    let mut seen = play(&mut engine, 200);
    remote.set_pattern(pattern(2, &[(0, Kick), (60, Snare)]));
    remote.set_kit(kit(100, 1.0)).unwrap();
    remote.mix().set_solo(Channel::Snare, true);
    remote.set_bpm(91.0);
    seen += play(&mut engine, 400);
    // The busy pattern, and the busy kit once its long sounds ended - so
    // handing a kit back was watched too.
    assert_eq!(remote.poll(), 2);
    remote.stop();
    seen += play(&mut engine, 100);
    remote.play();
    seen += play(&mut engine, 100);
    assert_eq!(seen, 0);
}

#[test]
fn the_watch_sees_an_allocation() {
    assert_eq!(allocations(|| drop(Vec::<u8>::with_capacity(8))), 2);
}

/// Counts allocations and frees on this thread while `run` runs.
fn allocations(run: impl FnOnce()) -> usize {
    SEEN.set(0);
    WATCHING.set(true);
    run();
    WATCHING.set(false);
    SEEN.get()
}

thread_local! {
    static WATCHING: Cell<bool> = const { Cell::new(false) };
    static SEEN: Cell<usize> = const { Cell::new(0) };
}

/// The system allocator, counting for [`allocations`].
struct Watch;

#[global_allocator]
static WATCH: Watch = Watch;

fn note() {
    let _ = WATCHING.try_with(|watching| {
        if watching.get() {
            let _ = SEEN.try_with(|seen| seen.set(seen.get() + 1));
        }
    });
}

unsafe impl GlobalAlloc for Watch {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        note();
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        note();
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        note();
        unsafe { System.realloc(ptr, layout, new_size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        note();
        unsafe { System.dealloc(ptr, layout) }
    }
}
