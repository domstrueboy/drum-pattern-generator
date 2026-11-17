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
