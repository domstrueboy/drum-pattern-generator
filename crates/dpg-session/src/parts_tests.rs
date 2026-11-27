use drum_patterns::{HitKind, Instrument};

use super::*;

fn parts() -> (Parts, StdRng) {
    let mut rng = StdRng::seed_from_u64(7);
    (Parts::new(&mut rng), rng)
}

fn ticks(pattern: &Pattern, instrument: Instrument, kind: Option<HitKind>) -> Vec<u32> {
    pattern
        .hits_of(instrument)
        .filter(|h| kind.is_none_or(|k| h.kind == k))
        .map(|h| h.tick)
        .collect()
}

fn ghosts(pattern: &Pattern) -> Vec<u32> {
    ticks(pattern, Instrument::Snare, Some(HitKind::Ghost))
}

#[test]
fn new_parts_play_the_default_groove() {
    let (parts, _) = parts();
    let laps = parts.build_laps();
    assert_eq!(laps.len(), 1);
    let p = &laps[0];
    assert_eq!(
        ticks(p, Instrument::HiHat, None),
        [0, 6, 12, 18, 24, 30, 36, 42]
    );
    assert_eq!(ticks(p, Instrument::Snare, None), [12, 36]);
    assert_eq!(ticks(p, Instrument::Kick, None).len(), 2);
}

#[test]
fn the_same_parts_build_the_same_laps() {
    let (parts, _) = parts();
    assert_eq!(parts.build_laps(), parts.clone().build_laps());
}

#[test]
fn bars_stay_in_range() {
    let (mut parts, _) = parts();
    parts.set_bars(9);
    assert_eq!(parts.bars, MAX_BARS);
    parts.set_bars(0);
    assert_eq!(parts.bars, 1);
    parts.set_bars(2);
    assert_eq!(parts.build_laps()[0].bars(), 2);
}

#[test]
fn generate_makes_ghost_notes_harder_up_to_the_top() {
    let (mut parts, mut rng) = parts();
    parts.set_ghosts(3);
    assert_eq!(parts.ghosts.level(), 1);
    for expected in [2, 3, 4, 4] {
        parts.generate_ghosts(&mut rng);
        assert_eq!(parts.ghosts.level(), expected);
    }
}

#[test]
fn a_new_ghost_count_starts_the_ladder_again() {
    let (mut parts, mut rng) = parts();
    parts.set_ghosts(2);
    parts.generate_ghosts(&mut rng);
    parts.set_ghosts(3);
    assert_eq!(parts.ghosts, Ladder::new(3));
}

#[test]
fn ghost_notes_move_only_the_ghost_notes() {
    let (mut parts, mut rng) = parts();
    parts.set_ghosts(2);
    let before = parts.build_laps().remove(0);
    parts.generate_ghosts(&mut rng);
    let after = parts.build_laps().remove(0);
    assert_ne!(ghosts(&before), ghosts(&after));
    let others = |p: &Pattern| {
        p.hits()
            .iter()
            .filter(|h| h.kind != HitKind::Ghost)
            .copied()
            .collect::<Vec<_>>()
    };
    assert_eq!(others(&before), others(&after));
}

#[test]
fn ghost_random_steps_through_its_set_and_back() {
    let (mut parts, _) = parts();
    parts.set_ghosts(2);
    parts.set_ghost_random_count(3);
    let first = ghosts(&parts.build_laps()[0]);
    let mut seen = vec![first.clone()];
    for _ in 0..2 {
        parts.ghost_random();
        seen.push(ghosts(&parts.build_laps()[0]));
    }
    parts.ghost_random();
    assert_eq!(parts.ghost_set.index(), 0);
    assert_eq!(ghosts(&parts.build_laps()[0]), first);
    assert!(seen[1..].iter().any(|g| *g != first));
}

#[test]
fn generate_keeps_the_random_count() {
    let (mut parts, mut rng) = parts();
    parts.set_ghost_random_count(9);
    parts.ghost_random();
    parts.generate_ghosts(&mut rng);
    assert_eq!(parts.ghost_set.count(), 9);
    assert_eq!(parts.ghost_set.index(), 0);
}

#[test]
fn hi_hat_random_turns_on_then_steps() {
    let (mut parts, _) = parts();
    parts.hi_hat_random();
    assert!(parts.hi_hat_random);
    assert_eq!(parts.hi_hat_set.index(), 0);
    parts.hi_hat_random();
    assert_eq!(parts.hi_hat_set.index(), 1);
}

#[test]
fn choosing_a_pulse_turns_random_off() {
    let (mut parts, _) = parts();
    parts.hi_hat_random();
    parts.set_pulse(NoteValue::Sixteenth);
    assert!(!parts.hi_hat_random);
    assert_eq!(
        ticks(&parts.build_laps()[0], Instrument::HiHat, None).len(),
        16
    );
}

#[test]
fn a_dotted_pulse_builds_its_laps_and_random_draws_each() {
    let (mut parts, _) = parts();
    parts.set_pulse(NoteValue::DottedEighth);
    assert_eq!(parts.build_laps().len(), 3);
    parts.hi_hat_random();
    parts.set_density(0);
    let laps = parts.build_laps();
    assert_eq!(laps.len(), 3);
    let hats: Vec<_> = laps
        .iter()
        .map(|p| ticks(p, Instrument::HiHat, None))
        .collect();
    assert_ne!(hats[0], hats[1]);
}

#[test]
fn choosing_accents_again_places_them_afresh() {
    let (mut parts, mut rng) = parts();
    parts.set_base_kicks(0);
    let accents = |parts: &Parts| {
        ticks(
            &parts.build_laps()[0],
            Instrument::Snare,
            Some(HitKind::Accent),
        )
    };
    parts.set_accents(2, &mut rng);
    let first = accents(&parts);
    assert_eq!(first.len(), 2);
    let differs = (0..8).any(|_| {
        parts.set_accents(2, &mut rng);
        accents(&parts) != first
    });
    assert!(differs);
}

#[test]
fn generate_moves_base_kicks_but_keeps_their_count() {
    let (mut parts, mut rng) = parts();
    parts.set_base_kicks(3);
    let kicks = |parts: &Parts| ticks(&parts.build_laps()[0], Instrument::Kick, None);
    let first = kicks(&parts);
    assert_eq!(first.len(), 3);
    let moved = (0..8).any(|_| {
        parts.generate_base_kicks(&mut rng);
        kicks(&parts) != first
    });
    assert!(moved);
    assert_eq!(kicks(&parts).len(), 3);
}

#[test]
fn extra_kicks_climb_their_own_ladder() {
    let (mut parts, mut rng) = parts();
    parts.set_extra_kicks(1);
    for expected in [2, 3, 3] {
        parts.generate_extra_kicks(&mut rng);
        assert_eq!(parts.extra_kicks.level(), expected);
    }
    let kicks = ticks(&parts.build_laps()[0], Instrument::Kick, None);
    assert!(kicks.iter().any(|t| t % 6 != 0), "{kicks:?}");
}

#[test]
fn counts_are_capped() {
    let (mut parts, mut rng) = parts();
    parts.set_accents(200, &mut rng);
    parts.set_ghosts(200);
    parts.set_density(200);
    parts.set_ghost_random_count(200);
    assert_eq!(parts.accents, MAX_COUNT);
    assert_eq!(parts.ghosts.count(), MAX_COUNT);
    assert_eq!(parts.density, 100);
    assert_eq!(parts.ghost_set.count(), Variants::MAX);
}
