use super::*;

/// Taps at each of `times`, returning what the last one gave.
fn taps(tap: &mut TapTempo, times: &[f64]) -> Option<f64> {
    times.iter().map(|&t| tap.tap(t)).last().flatten()
}

#[test]
fn the_first_tap_gives_nothing_the_second_a_tempo() {
    let mut tap = TapTempo::new();
    assert_eq!(tap.tap(10.0), None);
    assert_eq!(tap.tap(10.5), Some(120.0));
}

#[test]
fn an_unsteady_hand_is_averaged() {
    let mut tap = TapTempo::new();
    // Gaps 0.48, 0.52, 0.49, 0.51: 0.5 on average.
    assert_eq!(taps(&mut tap, &[0.0, 0.48, 1.0, 1.49, 2.0]), Some(120.0));
}

#[test]
fn only_the_last_four_gaps_count() {
    let mut tap = TapTempo::new();
    // Gaps of 0.6 s (100 BPM) slow to 0.7 s, a step small enough to be
    // averaged; four gaps later only the slower ones count.
    let mut times = vec![0.0, 0.6, 1.2];
    for i in 1..=4 {
        times.push(1.2 + 0.7 * f64::from(i));
    }
    assert_eq!(taps(&mut tap, &times), Some(86.0));
}

#[test]
fn a_clear_change_starts_afresh() {
    let mut tap = TapTempo::new();
    taps(&mut tap, &[0.0, 0.5, 1.0, 1.5]);
    // Twice as fast: the new tempo at once, not an average with the old.
    assert_eq!(tap.tap(1.75), Some(240.0));
}

#[test]
fn a_long_pause_starts_a_new_count() {
    let mut tap = TapTempo::new();
    taps(&mut tap, &[0.0, 0.5, 1.0]);
    let back = 1.0 + TAP_TIMEOUT + 0.1;
    assert_eq!(tap.tap(back), None);
    assert_eq!(tap.tap(back + 1.0), Some(60.0));
}

#[test]
fn the_tempo_is_whole_and_in_range() {
    let mut tap = TapTempo::new();
    assert_eq!(taps(&mut tap, &[0.0, 0.123]), Some(BPM_MAX), "488 BPM");
    let mut tap = TapTempo::new();
    assert_eq!(taps(&mut tap, &[0.0, 0.65]), Some(92.0), "92.3 BPM");
}

#[test]
fn the_slowest_tempo_can_be_tapped() {
    let mut tap = TapTempo::new();
    assert_eq!(taps(&mut tap, &[0.0, 2.0, 4.0]), Some(BPM_MIN));
}
