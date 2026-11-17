use rand::SeedableRng;
use rand::rngs::StdRng;

use super::*;
use crate::kick::base_kicks;
use crate::snare::{Feel, snare};
use crate::time::TimeSignature;

fn rng(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}

/// Basic snare with 2 accents, 2 base kicks a bar, then ghost notes.
fn groove(bars: u8, ghosts: u8, level: u8, seed: u64) -> Pattern {
    let mut p = Pattern::new(TimeSignature::FOUR_FOUR, bars).unwrap();
    snare(&mut p, Feel::Basic, 2, &mut rng(seed));
    base_kicks(&mut p, 2, &mut rng(seed + 1000));
    ghost_notes(&mut p, ghosts, level, &mut rng(seed + 2000));
    p
}

fn snare_of(p: &Pattern, kind: HitKind) -> Vec<u32> {
    p.hits_of(Instrument::Snare)
        .filter(|h| h.kind == kind)
        .map(|h| h.tick)
        .collect()
}

fn ghosts(p: &Pattern) -> Vec<u32> {
    snare_of(p, HitKind::Ghost)
}

fn kicks(p: &Pattern) -> Vec<u32> {
    p.hits_of(Instrument::Kick).map(|h| h.tick).collect()
}

fn near(p: &Pattern, tick: u32, others: &[u32]) -> bool {
    ladder::Neighbours::new(p).any(tick, others)
}

fn weak(tick: u32) -> bool {
    tick % 6 == 3
}

#[test]
fn places_the_asked_ghosts_in_every_bar() {
    for level in 1..=GHOST_LEVELS {
        for seed in 0..20 {
            let p = groove(2, 2, level, seed);
            for bar in 0..2 {
                let in_bar = ghosts(&p).iter().filter(|&&t| t / 48 == bar).count();
                assert_eq!(in_bar, 2, "level {level}, seed {seed}, bar {bar}");
            }
        }
    }
}

#[test]
fn ghosts_sit_on_16ths_off_the_other_snare_hits() {
    for level in 1..=GHOST_LEVELS {
        for seed in 0..30 {
            let p = groove(1, 3, level, seed);
            assert!(
                ghosts(&p).iter().all(|t| t % 3 == 0),
                "level {level}, seed {seed}"
            );
            // One snare hit per tick: a ghost never replaced a main hit or accent.
            assert_eq!(
                snare_of(&p, HitKind::Normal),
                [12, 36],
                "level {level}, seed {seed}"
            );
            assert_eq!(
                snare_of(&p, HitKind::Accent).len(),
                2,
                "level {level}, seed {seed}"
            );
        }
    }
}

#[test]
fn level_one_keeps_to_weak_16ths_away_from_accents_and_off_kicks() {
    for seed in 0..50 {
        let p = groove(2, 3, 1, seed);
        let (accents, kicks) = (snare_of(&p, HitKind::Accent), kicks(&p));
        for t in ghosts(&p) {
            assert!(weak(t), "seed {seed}: {t} is not a weak 16th");
            assert!(
                !near(&p, t, &accents),
                "seed {seed}: {t} is next to an accent"
            );
            assert!(!kicks.contains(&t), "seed {seed}: {t} is on a kick");
        }
    }
}

#[test]
fn level_two_adds_a_ghost_next_to_an_accent() {
    for seed in 0..50 {
        let p = groove(1, 2, 2, seed);
        let accents = snare_of(&p, HitKind::Accent);
        let g = ghosts(&p);
        assert!(g.iter().all(|&t| weak(t)), "seed {seed}: {g:?}");
        assert!(
            g.iter().any(|&t| near(&p, t, &accents)),
            "seed {seed}: {g:?}"
        );
    }
}

#[test]
fn level_three_adds_a_ghost_on_an_8th_but_none_on_a_kick() {
    for seed in 0..50 {
        let p = groove(1, 2, 3, seed);
        let kicks = kicks(&p);
        let g = ghosts(&p);
        assert!(
            g.iter().all(|t| !kicks.contains(t)),
            "seed {seed}: ghost on a kick"
        );
        assert!(g.iter().any(|&t| !weak(t)), "seed {seed}: {g:?}");
    }
}

#[test]
fn level_four_puts_a_ghost_on_a_kick() {
    for seed in 0..50 {
        let p = groove(1, 2, 4, seed);
        let kicks = kicks(&p);
        assert!(ghosts(&p).iter().any(|t| kicks.contains(t)), "seed {seed}");
    }
}

#[test]
fn levels_outside_the_ladder_are_clamped() {
    assert_eq!(groove(1, 2, 0, 5), groove(1, 2, 1, 5));
    assert_eq!(groove(1, 2, 9, 5), groove(1, 2, GHOST_LEVELS, 5));
}

#[test]
fn generating_again_replaces_only_the_ghosts() {
    let mut p = groove(1, 3, 2, 1);
    let normal = snare_of(&p, HitKind::Normal);
    let accents = snare_of(&p, HitKind::Accent);
    let kicks_before = kicks(&p);

    ghost_notes(&mut p, 1, 1, &mut rng(42));

    assert_eq!(ghosts(&p).len(), 1);
    assert_eq!(snare_of(&p, HitKind::Normal), normal);
    assert_eq!(snare_of(&p, HitKind::Accent), accents);
    assert_eq!(kicks(&p), kicks_before);
}

#[test]
fn the_same_seed_writes_the_same_ghosts() {
    assert_eq!(groove(4, 3, 3, 8), groove(4, 3, 3, 8));
}

#[test]
fn the_loop_point_counts_as_next_to() {
    // The bar's last 16th sits next to its first, since the pattern repeats.
    let p = Pattern::new(TimeSignature::FOUR_FOUR, 1).unwrap();
    assert!(near(&p, 45, &[0]));
    assert!(near(&p, 3, &[0]));
    assert!(!near(&p, 42, &[0]));
}
