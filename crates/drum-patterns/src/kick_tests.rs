use std::collections::HashSet;

use rand::SeedableRng;
use rand::rngs::StdRng;

use super::*;
use crate::snare::{Feel, snare};
use crate::time::TimeSignature;

fn rng(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}

/// A pattern with a basic snare and two accents a bar, then base kicks.
fn groove(bars: u8, kicks: u8, seed: u64) -> Pattern {
    let mut p = Pattern::new(TimeSignature::FOUR_FOUR, bars).unwrap();
    snare(&mut p, Feel::Basic, 2, &mut rng(seed));
    base_kicks(&mut p, kicks, &mut rng(seed + 1000));
    p
}

fn kick_ticks(p: &Pattern) -> Vec<u32> {
    p.hits_of(Instrument::Kick).map(|h| h.tick).collect()
}

#[test]
fn places_the_asked_kicks_in_every_bar() {
    for count in 1..=4 {
        for seed in 0..20 {
            let p = groove(3, count, seed);
            for bar in 0..3 {
                let in_bar = kick_ticks(&p).iter().filter(|&&t| t / 48 == bar).count();
                assert_eq!(
                    in_bar, count as usize,
                    "{count} kicks, seed {seed}, bar {bar}"
                );
            }
        }
    }
}

#[test]
fn kicks_sit_on_8ths_the_snare_leaves_free() {
    for seed in 0..50 {
        let p = groove(2, 4, seed);
        for tick in kick_ticks(&p) {
            assert_eq!(tick % 6, 0, "seed {seed}: {tick} is off the 8ths");
            assert_eq!(
                p.get(tick, Instrument::Snare),
                None,
                "seed {seed}: kick on the snare at {tick}"
            );
        }
    }
}

#[test]
fn more_kicks_than_free_8ths_fill_them_all() {
    // Basic plus 2 accents leaves 4 of the 8 eighths free.
    let p = groove(1, 8, 3);
    assert_eq!(kick_ticks(&p).len(), 4);
}

#[test]
fn kicks_are_normal_hits() {
    let p = groove(2, 3, 5);
    assert!(
        p.hits_of(Instrument::Kick)
            .all(|h| h.kind == HitKind::Normal)
    );
}

#[test]
fn generating_again_replaces_the_base_kicks() {
    let mut p = groove(1, 3, 1);
    base_kicks(&mut p, 1, &mut rng(99));
    assert_eq!(kick_ticks(&p).len(), 1);
}

#[test]
fn extra_kicks_between_the_8ths_stay() {
    let mut p = Pattern::new(TimeSignature::FOUR_FOUR, 1).unwrap();
    p.set(Hit::new(9, Instrument::Kick, HitKind::Normal))
        .unwrap();
    p.set(Hit::new(0, Instrument::Kick, HitKind::Normal))
        .unwrap();

    base_kicks(&mut p, 0, &mut rng(1));

    assert_eq!(kick_ticks(&p), [9]);
}

#[test]
fn other_instruments_stay() {
    let p = groove(1, 2, 7);
    let snare_before = {
        let mut q = Pattern::new(TimeSignature::FOUR_FOUR, 1).unwrap();
        snare(&mut q, Feel::Basic, 2, &mut rng(7));
        q.hits().to_vec()
    };
    let snare_after: Vec<Hit> = p.hits_of(Instrument::Snare).copied().collect();
    assert_eq!(snare_after, snare_before);
}

#[test]
fn the_same_seed_writes_the_same_kicks() {
    assert_eq!(groove(4, 2, 11), groove(4, 2, 11));
}

#[test]
fn different_seeds_vary_the_kicks() {
    let mut p = groove(1, 0, 0);
    let variants: HashSet<Vec<u32>> = (0..20)
        .map(|seed| {
            base_kicks(&mut p, 2, &mut rng(seed));
            kick_ticks(&p)
        })
        .collect();
    assert!(variants.len() > 3, "only {} variants", variants.len());
}

/// Basic snare with 2 accents, 2 base kicks a bar, then extra kicks.
fn with_extras(bars: u8, extras: u8, level: u8, seed: u64) -> Pattern {
    let mut p = groove(bars, 2, seed);
    extra_kicks(&mut p, extras, level, &mut rng(seed + 2000));
    p
}

fn extras(p: &Pattern) -> Vec<u32> {
    kick_ticks(p).into_iter().filter(|t| t % 6 != 0).collect()
}

fn accents(p: &Pattern) -> Vec<u32> {
    p.hits_of(Instrument::Snare)
        .filter(|h| h.kind == HitKind::Accent)
        .map(|h| h.tick)
        .collect()
}

fn near(p: &Pattern, tick: u32, others: &[u32]) -> bool {
    crate::ladder::Neighbours::new(p).any(tick, others)
}

#[test]
fn extra_kicks_sit_between_the_8ths_off_the_snare() {
    for level in 1..=EXTRA_KICK_LEVELS {
        for seed in 0..30 {
            let p = with_extras(2, 2, level, seed);
            let e = extras(&p);
            assert!(e.len() <= 4, "level {level}, seed {seed}: {e:?}");
            assert!(
                e.iter()
                    .all(|&t| t % 6 == 3 && p.get(t, Instrument::Snare).is_none())
            );
        }
    }
}

#[test]
fn from_level_two_there_is_room_for_the_asked_extras() {
    for level in 2..=EXTRA_KICK_LEVELS {
        for seed in 0..30 {
            let p = with_extras(2, 2, level, seed);
            for bar in 0..2 {
                let in_bar = extras(&p).iter().filter(|&&t| t / 48 == bar).count();
                assert_eq!(in_bar, 2, "level {level}, seed {seed}, bar {bar}");
            }
        }
    }
}

#[test]
fn level_one_extra_kicks_are_the_a_before_a_beat() {
    for seed in 0..50 {
        let p = with_extras(1, 2, 1, seed);
        let accents = accents(&p);
        let e = extras(&p);
        assert!(!e.is_empty(), "seed {seed}");
        for t in e {
            assert_eq!(t % 12, 9, "seed {seed}: {t} is not an `a`");
            assert!(
                !near(&p, t, &accents),
                "seed {seed}: {t} is next to an accent"
            );
        }
    }
}

#[test]
fn level_two_adds_an_e() {
    for seed in 0..50 {
        let p = with_extras(1, 2, 2, seed);
        let accents = accents(&p);
        let e = extras(&p);
        assert!(
            e.iter().all(|&t| !near(&p, t, &accents)),
            "seed {seed}: {e:?}"
        );
        assert!(
            e.iter().any(|t| t % 12 == 3),
            "seed {seed}: no `e` in {e:?}"
        );
    }
}

#[test]
fn level_three_adds_one_next_to_an_accent() {
    for seed in 0..50 {
        let p = with_extras(1, 2, 3, seed);
        let accents = accents(&p);
        assert!(
            extras(&p).iter().any(|&t| near(&p, t, &accents)),
            "seed {seed}"
        );
    }
}

#[test]
fn extra_kicks_keep_the_base_kicks() {
    let base = kick_ticks(&groove(1, 2, 4));
    let p = with_extras(1, 3, 2, 4);
    let on_8ths: Vec<u32> = kick_ticks(&p).into_iter().filter(|t| t % 6 == 0).collect();
    assert_eq!(on_8ths, base);
}

#[test]
fn generating_extras_again_replaces_them() {
    let mut p = with_extras(1, 3, 2, 6);
    extra_kicks(&mut p, 1, 2, &mut rng(77));
    assert_eq!(extras(&p).len(), 1);
}

#[test]
fn extra_kick_levels_outside_the_ladder_are_clamped() {
    assert_eq!(with_extras(1, 2, 0, 3), with_extras(1, 2, 1, 3));
    assert_eq!(
        with_extras(1, 2, 7, 3),
        with_extras(1, 2, EXTRA_KICK_LEVELS, 3)
    );
}
