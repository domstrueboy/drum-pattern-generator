use drum_patterns::TICKS_PER_QUARTER;

use super::*;

const FOUR_FOUR: TimeSignature = TimeSignature::FOUR_FOUR;

/// Every tick's absolute frame, playing `total` frames in blocks of the
/// given sizes, repeated.
fn tick_frames(clock: &mut Clock, total: usize, blocks: &[usize]) -> Vec<(u64, u64)> {
    let mut out = Vec::new();
    let mut played = 0;
    for &size in blocks.iter().cycle() {
        if played >= total {
            break;
        }
        let size = size.min(total - played);
        let start = clock.frame();
        clock.advance(size, |t| out.push((t.tick, start + t.offset as u64)));
        played += size;
    }
    out
}

#[test]
fn at_120_bpm_a_tick_is_2000_frames_at_48k() {
    // 120 beats a minute, 12 ticks a beat: 24 ticks a second.
    let mut clock = Clock::new(48_000, FOUR_FOUR, 120.0);
    let ticks = tick_frames(&mut clock, 10_000, &[10_000]);
    assert_eq!(ticks, [(0, 0), (1, 2000), (2, 4000), (3, 6000), (4, 8000)]);
}

#[test]
fn block_sizes_change_nothing() {
    let one = tick_frames(
        &mut Clock::new(44_100, FOUR_FOUR, 92.0),
        300_000,
        &[300_000],
    );
    let odd = tick_frames(
        &mut Clock::new(44_100, FOUR_FOUR, 92.0),
        300_000,
        &[1, 255, 4096, 7, 333],
    );
    assert_eq!(one, odd);
    // Exactly once each, in order.
    assert!(
        one.iter()
            .enumerate()
            .all(|(i, &(tick, _))| tick == i as u64)
    );
}

#[test]
fn the_grid_does_not_drift_over_a_thousand_bars() {
    let rate = 44_100;
    let mut clock = Clock::new(rate, FOUR_FOUR, 92.0);
    let bar = u64::from(FOUR_FOUR.ticks_per_bar());
    let frames_per_tick = f64::from(rate) * 60.0 / 92.0 / f64::from(TICKS_PER_QUARTER);
    let total = (1001.0 * bar as f64 * frames_per_tick) as usize;
    let ticks = tick_frames(&mut clock, total, &[256]);
    let target = 1000 * bar;
    let (_, frame) = ticks[target as usize];
    assert_eq!(frame, (target as f64 * frames_per_tick).round() as u64);
}

#[test]
fn a_tempo_change_carries_the_beat_on() {
    let mut clock = Clock::new(48_000, FOUR_FOUR, 120.0);
    // Halfway between tick 2 (4000) and tick 3 (6000), go twice as fast.
    let before = tick_frames(&mut clock, 5_000, &[5_000]);
    clock.set_bpm(240.0);
    let after = tick_frames(&mut clock, 5_000, &[5_000]);
    assert_eq!(before, [(0, 0), (1, 2000), (2, 4000)]);
    // Half a tick left, now 1000 frames a tick: tick 3 at 5500.
    assert_eq!(
        after,
        [(3, 5500), (4, 6500), (5, 7500), (6, 8500), (7, 9500)]
    );
}

#[test]
fn many_tempo_changes_skip_and_repeat_nothing() {
    let mut clock = Clock::new(48_000, FOUR_FOUR, 100.0);
    let mut ticks = Vec::new();
    for i in 0..200 {
        clock.set_bpm(30.0 + (i * 37 % 270) as f64);
        ticks.extend(tick_frames(&mut clock, 3_000, &[256]));
    }
    assert!(
        ticks
            .iter()
            .enumerate()
            .all(|(i, &(tick, _))| tick == i as u64)
    );
    assert!(ticks.windows(2).all(|w| w[0].1 <= w[1].1));
}

#[test]
fn the_tempo_is_kept_in_range() {
    assert_eq!(Clock::new(48_000, FOUR_FOUR, 5.0).bpm(), BPM_MIN);
    assert_eq!(Clock::new(48_000, FOUR_FOUR, 900.0).bpm(), BPM_MAX);
}

#[test]
fn in_six_eight_the_beat_is_an_eighth() {
    // 120 eighths a minute: 2 a second, 6 ticks each, 4000 frames a tick.
    let mut clock = Clock::new(48_000, TimeSignature::new(6, 8).unwrap(), 120.0);
    let ticks = tick_frames(&mut clock, 8_001, &[8_001]);
    assert_eq!(ticks, [(0, 0), (1, 4000), (2, 8000)]);
}

#[test]
fn the_position_is_exact_between_ticks() {
    let mut clock = Clock::new(48_000, FOUR_FOUR, 120.0);
    clock.advance(5_000, |_| {});
    assert_eq!(clock.tick_at(clock.frame() as f64), 2.5);
    // 1000 frames of latency back: what is heard now.
    assert_eq!(clock.tick_at(clock.frame() as f64 - 1000.0), 2.0);
}

#[test]
fn bars_of_playback_count_on() {
    let clock = Clock::new(48_000, FOUR_FOUR, 120.0);
    assert_eq!(clock.bar_of(0), 0);
    assert_eq!(clock.bar_of(47), 0);
    assert_eq!(clock.bar_of(48), 1);
    assert_eq!(clock.bar_of(48 * 1000 + 5), 1000);
}
