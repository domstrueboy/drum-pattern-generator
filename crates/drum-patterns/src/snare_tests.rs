use std::collections::HashSet;

use rand::SeedableRng;
use rand::rngs::StdRng;

use super::*;
use crate::time::TimeSignature;

fn written(bars: u8, feel: Feel, accents: u8, seed: u64) -> Pattern {
    let mut p = Pattern::new(TimeSignature::FOUR_FOUR, bars).unwrap();
    snare(&mut p, feel, accents, &mut StdRng::seed_from_u64(seed));
    p
}

fn ticks_of(p: &Pattern, kind: HitKind) -> Vec<u32> {
    p.hits_of(Instrument::Snare)
        .filter(|h| h.kind == kind)
        .map(|h| h.tick)
        .collect()
}

#[test]
fn basic_puts_the_snare_on_two_and_four() {
    let p = written(1, Feel::Basic, 0, 1);
    assert_eq!(ticks_of(&p, HitKind::Normal), [12, 36]);
}

#[test]
fn half_time_puts_the_snare_on_three() {
    let p = written(1, Feel::HalfTime, 0, 1);
    assert_eq!(ticks_of(&p, HitKind::Normal), [24]);
}

#[test]
fn double_time_puts_the_snare_on_every_and() {
    let p = written(1, Feel::DoubleTime, 0, 1);
    assert_eq!(ticks_of(&p, HitKind::Normal), [6, 18, 30, 42]);
}

#[test]
fn every_bar_gets_the_main_hits() {
    let p = written(2, Feel::Basic, 0, 1);
    assert_eq!(ticks_of(&p, HitKind::Normal), [12, 36, 60, 84]);
}

#[test]
fn places_the_asked_accents_in_every_bar() {
    for accents in [2, 3, 4] {
        for seed in 0..20 {
            let p = written(3, Feel::Basic, accents, seed);
            for bar in 0..3 {
                let in_bar = ticks_of(&p, HitKind::Accent)
                    .into_iter()
                    .filter(|t| t / 48 == bar)
                    .count();
                assert_eq!(
                    in_bar, accents as usize,
                    "{accents} accents, seed {seed}, bar {bar}"
                );
            }
        }
    }
}

#[test]
fn accents_sit_on_free_8ths() {
    for feel in [Feel::Basic, Feel::HalfTime, Feel::DoubleTime] {
        for seed in 0..50 {
            let p = written(2, feel, 4, seed);
            let main = ticks_of(&p, HitKind::Normal);
            for tick in ticks_of(&p, HitKind::Accent) {
                assert_eq!(tick % 6, 0, "{feel:?} seed {seed}: {tick} is off the 8ths");
                assert!(
                    !main.contains(&tick),
                    "{feel:?} seed {seed}: {tick} is a main hit"
                );
            }
        }
    }
}

#[test]
fn accents_beyond_the_free_8ths_fill_them_all() {
    // Double time leaves 2, 3 and 4 free; the 1 is the kick's.
    let p = written(1, Feel::DoubleTime, 6, 1);
    assert_eq!(ticks_of(&p, HitKind::Accent), [12, 24, 36]);
}

#[test]
fn accents_leave_the_one_to_the_kick() {
    for feel in [Feel::Basic, Feel::HalfTime, Feel::DoubleTime] {
        for seed in 0..50 {
            let p = written(2, feel, 4, seed);
            for tick in ticks_of(&p, HitKind::Accent) {
                assert_ne!(tick % 48, 0, "{feel:?} seed {seed}: an accent on the 1");
            }
        }
    }
}

#[test]
fn a_single_accent_is_placed() {
    for seed in 0..20 {
        assert_eq!(
            ticks_of(&written(2, Feel::Basic, 1, seed), HitKind::Accent).len(),
            2
        );
    }
}

#[test]
fn the_same_seed_writes_the_same_snare() {
    assert_eq!(written(4, Feel::Basic, 3, 7), written(4, Feel::Basic, 3, 7));
}

#[test]
fn different_seeds_vary_the_accents() {
    let variants: HashSet<Vec<u32>> = (0..20)
        .map(|seed| ticks_of(&written(1, Feel::Basic, 2, seed), HitKind::Accent))
        .collect();
    assert!(variants.len() > 5, "only {} variants", variants.len());
}

fn with_kicks(ticks: &[u32]) -> Pattern {
    let mut p = Pattern::new(TimeSignature::FOUR_FOUR, 2).unwrap();
    for &tick in ticks {
        p.set(Hit::new(tick, Instrument::Kick, HitKind::Normal))
            .unwrap();
    }
    p
}

#[test]
fn accents_avoid_the_kicks() {
    let kicks = [0, 24, 30, 48, 66];
    for seed in 0..50 {
        let mut p = with_kicks(&kicks);
        snare(&mut p, Feel::Basic, 4, &mut StdRng::seed_from_u64(seed));
        for tick in ticks_of(&p, HitKind::Accent) {
            assert!(
                !kicks.contains(&tick),
                "seed {seed}: accent on the kick at {tick}"
            );
        }
    }
}

#[test]
fn kicks_on_every_free_8th_leave_no_room_for_accents() {
    // Basic leaves 1, 1и, 2и, 3, 3и, 4и free in bar 1.
    let mut p = with_kicks(&[0, 6, 18, 24, 30, 42]);
    snare(&mut p, Feel::Basic, 4, &mut StdRng::seed_from_u64(1));
    assert!(ticks_of(&p, HitKind::Accent).iter().all(|&t| t >= 48));
    assert_eq!(
        ticks_of(&p, HitKind::Accent).len(),
        4,
        "bar 2 still has room"
    );
}

#[test]
fn rewrites_the_snare_only() {
    let mut p = Pattern::new(TimeSignature::FOUR_FOUR, 1).unwrap();
    p.set(Hit::new(3, Instrument::Snare, HitKind::Ghost))
        .unwrap();
    p.set(Hit::new(0, Instrument::Kick, HitKind::Normal))
        .unwrap();

    snare(&mut p, Feel::HalfTime, 0, &mut StdRng::seed_from_u64(1));

    assert_eq!(p.get(3, Instrument::Snare), None);
    assert!(p.get(0, Instrument::Kick).is_some());
}

#[test]
fn defaults_to_basic() {
    assert_eq!(Feel::default(), Feel::Basic);
}
