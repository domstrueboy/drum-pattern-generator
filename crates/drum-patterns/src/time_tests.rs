use super::*;

#[test]
fn four_four_has_48_ticks_a_bar() {
    let t = TimeSignature::FOUR_FOUR;
    assert_eq!(t.ticks_per_beat(), 12);
    assert_eq!(t.ticks_per_bar(), 48);
}

#[test]
fn three_four_and_six_eight_have_36_ticks_a_bar() {
    let three_four = TimeSignature::new(3, 4).unwrap();
    let six_eight = TimeSignature::new(6, 8).unwrap();
    assert_eq!(three_four.ticks_per_bar(), 36);
    assert_eq!(six_eight.ticks_per_beat(), 6);
    assert_eq!(six_eight.ticks_per_bar(), 36);
}

#[test]
fn half_and_sixteenth_units_divide_the_grid() {
    assert_eq!(TimeSignature::new(2, 2).unwrap().ticks_per_beat(), 24);
    assert_eq!(TimeSignature::new(7, 16).unwrap().ticks_per_bar(), 21);
}

#[test]
fn rejects_units_the_grid_cannot_hold() {
    for unit in [0, 1, 3, 32] {
        assert_eq!(TimeSignature::new(4, unit), None, "unit {unit}");
    }
}

#[test]
fn rejects_zero_or_too_many_beats() {
    assert_eq!(TimeSignature::new(0, 4), None);
    assert_eq!(TimeSignature::new(17, 4), None);
    assert!(TimeSignature::new(16, 4).is_some());
}

#[test]
fn defaults_to_four_four() {
    assert_eq!(TimeSignature::default(), TimeSignature::FOUR_FOUR);
}

#[test]
fn displays_as_written() {
    assert_eq!(TimeSignature::new(6, 8).unwrap().to_string(), "6/8");
}

#[test]
fn note_values_match_the_spec_table() {
    use NoteValue::*;
    let table = [
        (Quarter, 12),
        (DottedEighth, 9),
        (Eighth, 6),
        (QuarterTriplet, 8),
        (EighthTriplet, 4),
        (Sixteenth, 3),
        (SixteenthTriplet, 2),
        (DottedQuarter, 18),
    ];
    for (value, ticks) in table {
        assert_eq!(value.ticks(), ticks, "{value:?}");
    }
}

#[test]
fn all_lists_every_value_longest_first() {
    let ticks: Vec<u32> = NoteValue::ALL.iter().map(|v| v.ticks()).collect();
    assert_eq!(ticks, [18, 12, 9, 8, 6, 4, 3, 2]);
}

#[test]
fn only_triplets_are_triplets() {
    let triplets: Vec<NoteValue> = NoteValue::ALL
        .into_iter()
        .filter(|v| v.is_triplet())
        .collect();
    assert_eq!(
        triplets,
        [
            NoteValue::QuarterTriplet,
            NoteValue::EighthTriplet,
            NoteValue::SixteenthTriplet
        ]
    );
}
