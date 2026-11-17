use super::*;
use crate::pattern::{HitKind, Instrument};

fn count(p: &Pattern, instrument: Instrument, kind: HitKind) -> usize {
    p.hits_of(instrument).filter(|h| h.kind == kind).count()
}

fn ticks(p: &Pattern, instrument: Instrument) -> Vec<u32> {
    p.hits_of(instrument).map(|h| h.tick).collect()
}

fn busy() -> Groove {
    Groove {
        bars: 2,
        hi_hat: NoteValue::Sixteenth,
        hi_hat_random: Some(50),
        accents: 2,
        ghosts: 2,
        ghost_level: 2,
        base_kicks: 2,
        extra_kicks: 1,
        extra_kick_level: 2,
        seed: 7,
        ..Groove::default()
    }
}

#[test]
fn the_default_is_a_plain_rock_bar() {
    let p = Groove::default().build().unwrap();
    assert_eq!(p.bars(), 1);
    assert_eq!(ticks(&p, Instrument::HiHat), [0, 6, 12, 18, 24, 30, 36, 42]);
    assert_eq!(ticks(&p, Instrument::Snare), [12, 36]);
    assert_eq!(p.hits_of(Instrument::Kick).count(), 2);
}

#[test]
fn every_part_gets_what_it_asks_for() {
    for seed in 0..30 {
        let p = Groove { seed, ..busy() }.build().unwrap();
        // Two bars of: 2 main hits, 2 accents, 2 ghosts, 2 + 1 kicks, 4 beats.
        assert_eq!(
            count(&p, Instrument::Snare, HitKind::Normal),
            4,
            "seed {seed}"
        );
        assert_eq!(
            count(&p, Instrument::Snare, HitKind::Accent),
            4,
            "seed {seed}"
        );
        assert_eq!(
            count(&p, Instrument::Snare, HitKind::Ghost),
            4,
            "seed {seed}"
        );
        assert_eq!(p.hits_of(Instrument::Kick).count(), 6, "seed {seed}");
        assert!(p.hits_of(Instrument::HiHat).count() >= 8, "seed {seed}");
    }
}

#[test]
fn kicks_and_snare_never_share_a_tick() {
    for seed in 0..30 {
        let p = Groove { seed, ..busy() }.build().unwrap();
        let snare = ticks(&p, Instrument::Snare);
        assert!(
            ticks(&p, Instrument::Kick)
                .iter()
                .all(|t| !snare.contains(t)),
            "seed {seed}"
        );
    }
}

#[test]
fn the_same_groove_builds_the_same_pattern() {
    assert_eq!(busy().build(), busy().build());
}

#[test]
fn another_seed_builds_another_pattern() {
    let other = Groove { seed: 8, ..busy() };
    assert_ne!(busy().build(), other.build());
}

#[test]
fn more_ghosts_leave_the_kicks_where_they_were() {
    // Ghosts come after the kicks, so they can't move them.
    let more = Groove {
        ghosts: 3,
        ..busy()
    };
    let kicks = |g: Groove| ticks(&g.build().unwrap(), Instrument::Kick);
    assert_eq!(kicks(busy()), kicks(more));
}

#[test]
fn a_pulse_ignores_the_seed() {
    let hats = |seed| {
        ticks(
            &Groove {
                seed,
                ..Groove::default()
            }
            .build()
            .unwrap(),
            Instrument::HiHat,
        )
    };
    assert_eq!(hats(1), hats(2));
}

#[test]
fn bars_out_of_range_are_an_error() {
    let g = Groove {
        bars: 0,
        ..Groove::default()
    };
    assert_eq!(g.build(), Err(PatternError::BarsOutOfRange(0)));
}
