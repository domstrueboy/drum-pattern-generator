use super::*;

fn one_bar() -> Pattern {
    Pattern::new(TimeSignature::FOUR_FOUR, 1).unwrap()
}

fn normal(tick: u32, instrument: Instrument) -> Hit {
    Hit::new(tick, instrument, HitKind::Normal)
}

#[test]
fn takes_one_to_four_bars() {
    for bars in 1..=MAX_BARS {
        assert!(
            Pattern::new(TimeSignature::FOUR_FOUR, bars).is_ok(),
            "{bars} bars"
        );
    }
}

#[test]
fn rejects_zero_and_five_bars() {
    for bars in [0, 5] {
        assert_eq!(
            Pattern::new(TimeSignature::FOUR_FOUR, bars),
            Err(PatternError::BarsOutOfRange(bars))
        );
    }
}

#[test]
fn length_is_bars_times_the_bar() {
    let four_four = Pattern::new(TimeSignature::FOUR_FOUR, 3).unwrap();
    let six_eight = Pattern::new(TimeSignature::new(6, 8).unwrap(), 2).unwrap();
    assert_eq!(four_four.len_ticks(), 144);
    assert_eq!(six_eight.len_ticks(), 72);
}

#[test]
fn starts_empty() {
    assert!(one_bar().hits().is_empty());
}

#[test]
fn keeps_hits_in_playing_order() {
    let mut p = one_bar();
    p.set(normal(24, Instrument::Kick)).unwrap();
    p.set(normal(12, Instrument::Snare)).unwrap();
    p.set(normal(0, Instrument::Kick)).unwrap();
    p.set(normal(0, Instrument::HiHat)).unwrap();
    p.set(normal(0, Instrument::Crash)).unwrap();

    let order: Vec<(u32, Instrument)> = p.hits().iter().map(|h| (h.tick, h.instrument)).collect();
    assert_eq!(
        order,
        [
            (0, Instrument::Crash),
            (0, Instrument::HiHat),
            (0, Instrument::Kick),
            (12, Instrument::Snare),
            (24, Instrument::Kick),
        ]
    );
}

#[test]
fn a_second_hit_on_the_same_tick_replaces_the_first() {
    let mut p = one_bar();
    p.set(normal(12, Instrument::Snare)).unwrap();
    p.set(Hit::new(12, Instrument::Snare, HitKind::Accent))
        .unwrap();

    assert_eq!(p.hits().len(), 1);
    assert_eq!(p.get(12, Instrument::Snare).unwrap().kind, HitKind::Accent);
}

#[test]
fn rejects_a_hit_past_the_end() {
    let mut p = one_bar();
    assert_eq!(
        p.set(normal(48, Instrument::Kick)),
        Err(PatternError::TickOutOfRange { tick: 48, len: 48 })
    );
    assert!(p.set(normal(47, Instrument::Kick)).is_ok());
}

#[test]
fn get_finds_only_that_instrument() {
    let mut p = one_bar();
    p.set(normal(6, Instrument::HiHat)).unwrap();
    assert!(p.get(6, Instrument::HiHat).is_some());
    assert_eq!(p.get(6, Instrument::Snare), None);
}

#[test]
fn remove_returns_the_hit() {
    let mut p = one_bar();
    let hit = normal(36, Instrument::Snare);
    p.set(hit).unwrap();

    assert_eq!(p.remove(36, Instrument::Snare), Some(hit));
    assert_eq!(p.remove(36, Instrument::Snare), None);
    assert!(p.hits().is_empty());
}

#[test]
fn hits_of_lists_one_instrument() {
    let mut p = one_bar();
    for tick in [0, 6, 12] {
        p.set(normal(tick, Instrument::HiHat)).unwrap();
    }
    p.set(normal(12, Instrument::Snare)).unwrap();

    let ticks: Vec<u32> = p.hits_of(Instrument::HiHat).map(|h| h.tick).collect();
    assert_eq!(ticks, [0, 6, 12]);
}

#[test]
fn clear_removes_only_that_instrument() {
    let mut p = one_bar();
    p.set(normal(0, Instrument::Kick)).unwrap();
    p.set(normal(0, Instrument::HiHat)).unwrap();
    p.set(normal(12, Instrument::Snare)).unwrap();

    p.clear(Instrument::HiHat);

    let left: Vec<Instrument> = p.hits().iter().map(|h| h.instrument).collect();
    assert_eq!(left, [Instrument::Kick, Instrument::Snare]);
}

#[test]
fn instruments_run_top_of_the_staff_first() {
    let mut sorted = Instrument::ALL;
    sorted.sort();
    assert_eq!(sorted, Instrument::ALL);
}

#[test]
fn retain_keeps_what_it_accepts() {
    let mut p = one_bar();
    for tick in [0, 3, 6, 9] {
        p.set(normal(tick, Instrument::Kick)).unwrap();
    }
    p.set(normal(6, Instrument::Snare)).unwrap();

    p.retain(|h| h.instrument != Instrument::Kick || h.tick % 6 != 0);

    let left: Vec<(u32, Instrument)> = p.hits().iter().map(|h| (h.tick, h.instrument)).collect();
    assert_eq!(
        left,
        [
            (3, Instrument::Kick),
            (6, Instrument::Snare),
            (9, Instrument::Kick)
        ]
    );
}
