use drum_patterns::{Groove, Hit, NoteValue, TimeSignature};

use super::*;

#[test]
fn a_bar_of_eighths_reads_one_column_an_eighth() {
    let mut p = Pattern::new(TimeSignature::FOUR_FOUR, 1).unwrap();
    for t in (0..48).step_by(6) {
        p.set(Hit::new(t, Instrument::HiHat, HitKind::Normal))
            .unwrap();
    }
    p.set(Hit::new(12, Instrument::Snare, HitKind::Normal))
        .unwrap();
    p.set(Hit::new(18, Instrument::Snare, HitKind::Ghost))
        .unwrap();
    p.set(Hit::new(0, Instrument::Kick, HitKind::Accent))
        .unwrap();
    let expected = "     1 2 3 4\n\
                    HH  |xxxxxxxx|\n\
                    SN  |--xg----|\n\
                    BD  |X-------|";
    assert_eq!(grid(&[p]), expected);
}

#[test]
fn a_dotted_hi_hat_shows_each_lap() {
    let groove = Groove {
        hi_hat: NoteValue::DottedEighth,
        base_kicks: 0,
        ..Groove::default()
    };
    let text = grid(&groove.build_laps().unwrap());
    let expected = "     1   2   3   4\n\
                    HH1 |x--x--x--x--x--x|\n\
                    HH2 |--x--x--x--x--x-|\n\
                    HH3 |-x--x--x--x--x--|\n\
                    SN  |----x-------x---|\n\
                    BD  |----------------|";
    assert_eq!(text, expected);
}

#[test]
fn bars_are_marked_off() {
    let p = Pattern::new(TimeSignature::FOUR_FOUR, 2).unwrap();
    assert!(grid(&[p]).ends_with("BD  |----|----|"));
}
