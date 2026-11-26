use super::*;

fn heard(skip: Option<BarSkip>, bars: u64) -> Vec<u64> {
    (0..bars).filter(|&b| BarSkip::allows(skip, b)).collect()
}

#[test]
fn two_one_plays_two_bars_then_rests_one() {
    assert_eq!(heard(BarSkip::new(2, 1), 9), [0, 1, 3, 4, 6, 7]);
}

#[test]
fn inverted_rests_longer_than_it_plays() {
    let one_two = BarSkip::new(2, 1).unwrap().inverted();
    assert_eq!((one_two.play(), one_two.skip()), (1, 2));
    assert_eq!(heard(Some(one_two), 9), [0, 3, 6]);
}

#[test]
fn off_plays_every_bar() {
    assert_eq!(heard(None, 4), [0, 1, 2, 3]);
}

#[test]
fn a_round_needs_bars_on_both_sides() {
    assert_eq!(BarSkip::new(0, 1), None);
    assert_eq!(BarSkip::new(8, 0), None);
}

#[test]
fn bar_skip_survives_an_atomic() {
    for skip in [None, BarSkip::new(1, 3), BarSkip::new(8, 8)] {
        assert_eq!(BarSkip::unpack(BarSkip::pack(skip)), skip);
    }
}

#[test]
fn the_crash_plays_once_or_every_few_bars() {
    let on = |crash: CrashRule| -> Vec<u64> { (0..17).filter(|&b| crash.plays(b)).collect() };
    assert_eq!(on(CrashRule::Once), [0]);
    assert_eq!(on(CrashRule::Every(4)), [0, 4, 8, 12, 16]);
    assert_eq!(on(CrashRule::Every(8)), [0, 8, 16]);
}

#[test]
fn the_crash_survives_an_atomic() {
    for crash in [CrashRule::Once, CrashRule::Every(4), CrashRule::Every(16)] {
        assert_eq!(CrashRule::unpack(crash.pack()), crash);
    }
}
