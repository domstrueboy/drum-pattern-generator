use super::*;
use crate::time::TimeSignature;

fn pulsed(time: TimeSignature, bars: u8, pulse: NoteValue) -> Pattern {
    let mut p = Pattern::new(time, bars).unwrap();
    hi_hat_pulse(&mut p, pulse);
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

    hi_hat_pulse(&mut p, NoteValue::Quarter);

    assert_eq!(hat_ticks(&p), [0, 12, 24, 36]);
    assert!(p.get(12, Instrument::Snare).is_some());
}

#[test]
fn six_eight_eighths_fill_six_per_bar() {
    let p = pulsed(TimeSignature::new(6, 8).unwrap(), 1, NoteValue::Eighth);
    assert_eq!(hat_ticks(&p), [0, 6, 12, 18, 24, 30]);
}
