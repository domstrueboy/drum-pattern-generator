use super::*;
use crate::time::TimeSignature;

fn pulsed(time: TimeSignature, bars: u8, pulse: NoteValue) -> Pattern {
    let mut p = Pattern::new(time, bars).unwrap();
    hi_hat_pulse(&mut p, pulse, 0);
    p
}

fn hat_ticks(p: &Pattern) -> Vec<u32> {
    p.hits_of(Instrument::HiHat).map(|h| h.tick).collect()
}

#[test]
fn eighths_land_on_every_8th() {
    let p = pulsed(TimeSignature::FOUR_FOUR, 1, NoteValue::Eighth);
    assert_eq!(hat_ticks(&p), [0, 6, 12, 18, 24, 30, 36, 42]);
}

#[test]
fn each_pulse_fills_a_four_four_bar() {
    use NoteValue::*;
    let per_bar = [
        (Quarter, 4),
        (Eighth, 8),
        (Sixteenth, 16),
        (QuarterTriplet, 6),
        (EighthTriplet, 12),
        (SixteenthTriplet, 24),
        (DottedQuarter, 3),
        (DottedEighth, 6),
    ];
    for (pulse, hits) in per_bar {
        let p = pulsed(TimeSignature::FOUR_FOUR, 1, pulse);
        assert_eq!(p.hits().len(), hits, "{pulse:?}");
    }
}

#[test]
fn dotted_quarters_run_on_across_the_barline() {
    let p = pulsed(TimeSignature::FOUR_FOUR, 2, NoteValue::DottedQuarter);
    assert_eq!(hat_ticks(&p), [0, 18, 36, 54, 72, 90]);
}

#[test]
fn dotted_pulses_line_up_again_after_three_bars() {
    for pulse in [NoteValue::DottedQuarter, NoteValue::DottedEighth] {
        let p = pulsed(TimeSignature::FOUR_FOUR, 3, pulse);
        let last = *hat_ticks(&p).last().unwrap();
        assert_eq!(last + pulse.ticks(), p.len_ticks(), "{pulse:?}");
    }
}

#[test]
fn a_dotted_pulse_runs_on_across_loops() {
    // One bar of dotted 8ths, three times through: the pulse never stops.
    let mut heard = Vec::new();
    for lap in 0..3 {
        let mut p = Pattern::new(TimeSignature::FOUR_FOUR, 1).unwrap();
        hi_hat_pulse(&mut p, NoteValue::DottedEighth, lap);
        heard.extend(hat_ticks(&p).into_iter().map(|t| t + 48 * lap));
    }
    let steady: Vec<u32> = (0..144).step_by(9).collect();
    assert_eq!(heard, steady);
}

#[test]
fn a_pulse_lines_up_with_the_pattern_after_its_laps() {
    use NoteValue::*;
    let p = |bars| Pattern::new(TimeSignature::FOUR_FOUR, bars).unwrap();
    for (bars, pulse, laps) in [
        (1, Eighth, 1),
        (1, SixteenthTriplet, 1),
        (1, DottedEighth, 3),
        (2, DottedQuarter, 3),
        (3, DottedEighth, 1),
        (4, DottedEighth, 3),
    ] {
        assert_eq!(
            hi_hat_laps(&p(bars), pulse),
            laps,
            "{bars} bars of {pulse:?}"
        );
        let mut first = p(bars);
        let mut again = p(bars);
        hi_hat_pulse(&mut first, pulse, 0);
        hi_hat_pulse(&mut again, pulse, laps);
        assert_eq!(first, again);
    }
}

#[test]
fn even_pulses_look_the_same_in_every_bar() {
    let p = pulsed(TimeSignature::FOUR_FOUR, 2, NoteValue::EighthTriplet);
    let ticks = hat_ticks(&p);
    let (first, second) = ticks.split_at(ticks.len() / 2);
    assert!(first.iter().zip(second).all(|(a, b)| a + 48 == *b));
}

#[test]
fn hits_are_normal_hi_hats() {
    let p = pulsed(TimeSignature::FOUR_FOUR, 4, NoteValue::Sixteenth);
    assert_eq!(p.hits().len(), 64);
    assert!(
        p.hits()
            .iter()
            .all(|h| h.instrument == Instrument::HiHat && h.kind == HitKind::Normal)
    );
}

#[test]
fn a_new_pulse_replaces_the_old_hi_hat_only() {
    let mut p = pulsed(TimeSignature::FOUR_FOUR, 1, NoteValue::Sixteenth);
    p.set(Hit::new(12, Instrument::Snare, HitKind::Normal))
        .unwrap();

    hi_hat_pulse(&mut p, NoteValue::Quarter, 0);

    assert_eq!(hat_ticks(&p), [0, 12, 24, 36]);
    assert!(p.get(12, Instrument::Snare).is_some());
}

#[test]
fn six_eight_eighths_fill_six_per_bar() {
    let p = pulsed(TimeSignature::new(6, 8).unwrap(), 1, NoteValue::Eighth);
    assert_eq!(hat_ticks(&p), [0, 6, 12, 18, 24, 30]);
}

mod random {
    use std::collections::HashSet;

    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::*;
    use crate::variants::Variants;

    fn random_hats(bars: u8, pulse: NoteValue, density: u8, seed: u64) -> Pattern {
        let mut p = Pattern::new(TimeSignature::FOUR_FOUR, bars).unwrap();
        hi_hat_random(&mut p, pulse, density, 0, &mut StdRng::seed_from_u64(seed));
        p
    }

    fn off_beats(p: &Pattern) -> usize {
        hat_ticks(p).iter().filter(|t| *t % 12 != 0).count()
    }

    #[test]
    fn hits_stay_on_the_pulse_grid() {
        for pulse in NoteValue::ALL {
            for seed in 0..20 {
                let p = random_hats(3, pulse, HI_HAT_DENSITY, seed);
                for t in hat_ticks(&p) {
                    assert_eq!(t % pulse.ticks(), 0, "{pulse:?} seed {seed}: {t}");
                }
            }
        }
    }

    #[test]
    fn every_beat_the_pulse_lands_on_keeps_its_hit() {
        for pulse in NoteValue::ALL {
            for seed in 0..20 {
                let p = random_hats(3, pulse, 0, seed);
                let hats = hat_ticks(&p);
                let beats: Vec<u32> = pulsed(TimeSignature::FOUR_FOUR, 3, pulse)
                    .hits()
                    .iter()
                    .map(|h| h.tick)
                    .filter(|t| t % 12 == 0)
                    .collect();
                assert_eq!(hats, beats, "{pulse:?} seed {seed}");
            }
        }
    }

    #[test]
    fn each_lap_is_random_on_its_own_grid() {
        let pulse = NoteValue::DottedEighth;
        for lap in 0..3 {
            let mut grid = Pattern::new(TimeSignature::FOUR_FOUR, 1).unwrap();
            hi_hat_pulse(&mut grid, pulse, lap);
            let mut p = Pattern::new(TimeSignature::FOUR_FOUR, 1).unwrap();
            hi_hat_random(&mut p, pulse, 0, lap, &mut StdRng::seed_from_u64(3));
            let on_grid = hat_ticks(&grid);
            let hats = hat_ticks(&p);
            assert!(hats.iter().all(|t| on_grid.contains(t)), "lap {lap}");
            let beats: Vec<_> = on_grid.into_iter().filter(|t| t % 12 == 0).collect();
            assert!(beats.iter().all(|t| hats.contains(t)), "lap {lap}");
        }
    }

    #[test]
    fn no_bar_is_left_empty() {
        for pulse in NoteValue::ALL {
            for seed in 0..50 {
                let p = random_hats(3, pulse, 0, seed);
                for bar in 0..3 {
                    let any = hat_ticks(&p).iter().any(|t| t / 48 == bar);
                    assert!(any, "{pulse:?} seed {seed}: bar {bar} is empty");
                }
            }
        }
    }

    #[test]
    fn a_bar_off_every_beat_still_keeps_a_hit() {
        // In 1/4 a dotted quarter skips the second bar's beat.
        let mut p = Pattern::new(TimeSignature::new(1, 4).unwrap(), 2).unwrap();
        hi_hat_random(
            &mut p,
            NoteValue::DottedQuarter,
            0,
            0,
            &mut StdRng::seed_from_u64(1),
        );
        assert_eq!(hat_ticks(&p), [0, 18]);
    }

    #[test]
    fn the_default_keeps_about_half_of_the_off_beats() {
        let kept: usize = (0..100)
            .map(|seed| off_beats(&random_hats(1, NoteValue::Sixteenth, HI_HAT_DENSITY, seed)))
            .sum();
        // 100 bars of 12 off-beat 16ths: about 600, comfortably inside 500-700.
        assert!((500..=700).contains(&kept), "kept {kept}");
    }

    #[test]
    fn density_sets_how_many_off_beats_stay() {
        let kept = |density| -> usize {
            (0..100)
                .map(|seed| off_beats(&random_hats(1, NoteValue::Sixteenth, density, seed)))
                .sum()
        };
        assert_eq!(kept(0), 0);
        assert!(kept(25) < kept(75), "{} vs {}", kept(25), kept(75));
        assert_eq!(kept(100), 1200);
        assert_eq!(kept(255), 1200);
    }

    #[test]
    fn variants_differ_and_rebuild_the_same() {
        let v = Variants::new(2026, 6);
        let build = |index: u8| {
            let mut p = Pattern::new(TimeSignature::FOUR_FOUR, 1).unwrap();
            let rng = &mut v.rng::<StdRng>(index);
            hi_hat_random(&mut p, NoteValue::Sixteenth, HI_HAT_DENSITY, 0, rng);
            hat_ticks(&p)
        };
        let all: HashSet<Vec<u32>> = (0..6).map(build).collect();
        assert_eq!(all.len(), 6);
        assert_eq!(build(4), build(4));
    }

    #[test]
    fn rewrites_the_hi_hat_only() {
        let mut p = random_hats(1, NoteValue::Sixteenth, 100, 1);
        p.set(Hit::new(12, Instrument::Snare, HitKind::Normal))
            .unwrap();
        hi_hat_random(
            &mut p,
            NoteValue::Quarter,
            0,
            0,
            &mut StdRng::seed_from_u64(2),
        );
        assert_eq!(hat_ticks(&p), [0, 12, 24, 36]);
        assert!(p.get(12, Instrument::Snare).is_some());
    }
}

fn accented(p: &Pattern) -> Vec<u32> {
    p.hits_of(Instrument::HiHat)
        .filter(|h| h.kind == HitKind::Accent)
        .map(|h| h.tick)
        .collect()
}

#[test]
fn accents_on_the_beats() {
    let mut p = pulsed(TimeSignature::FOUR_FOUR, 1, NoteValue::Sixteenth);
    hi_hat_accents(&mut p, HiHatAccents::Beats);
    assert_eq!(accented(&p), [0, 12, 24, 36]);
    assert_eq!(hat_ticks(&p).len(), 16);
}

#[test]
fn accents_on_the_offbeats() {
    let mut p = pulsed(TimeSignature::FOUR_FOUR, 1, NoteValue::Eighth);
    hi_hat_accents(&mut p, HiHatAccents::Offbeats);
    assert_eq!(accented(&p), [6, 18, 30, 42]);
}

#[test]
fn eighth_triplets_have_no_offbeat_to_accent() {
    let mut p = pulsed(TimeSignature::FOUR_FOUR, 1, NoteValue::EighthTriplet);
    hi_hat_accents(&mut p, HiHatAccents::Offbeats);
    assert!(accented(&p).is_empty());
}

#[test]
fn accents_off_makes_every_hit_normal_again() {
    let mut p = pulsed(TimeSignature::FOUR_FOUR, 1, NoteValue::Eighth);
    hi_hat_accents(&mut p, HiHatAccents::Beats);
    hi_hat_accents(&mut p, HiHatAccents::Off);
    assert!(accented(&p).is_empty());
}

#[test]
fn accents_leave_other_instruments_alone() {
    let mut p = pulsed(TimeSignature::FOUR_FOUR, 1, NoteValue::Eighth);
    p.set(Hit::new(0, Instrument::Snare, HitKind::Ghost))
        .unwrap();
    hi_hat_accents(&mut p, HiHatAccents::Beats);
    let snare: Vec<_> = p.hits_of(Instrument::Snare).collect();
    assert_eq!(snare[0].kind, HitKind::Ghost);
}
