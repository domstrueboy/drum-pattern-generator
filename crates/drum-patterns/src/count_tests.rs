use super::*;
use crate::time::TimeSignature;

fn labels(time: TimeSignature, bars: u8) -> Vec<CountLabel> {
    let pattern = Pattern::new(time, bars).unwrap();
    count(&pattern).into_iter().map(|m| m.label).collect()
}

use CountLabel::{And, Beat};

#[test]
fn four_four_counts_one_and_two_and() {
    assert_eq!(
        labels(TimeSignature::FOUR_FOUR, 1),
        [Beat(1), And, Beat(2), And, Beat(3), And, Beat(4), And]
    );
}

#[test]
fn marks_sit_on_every_8th() {
    let pattern = Pattern::new(TimeSignature::FOUR_FOUR, 1).unwrap();
    let ticks: Vec<u32> = count(&pattern).iter().map(|m| m.tick).collect();
    assert_eq!(ticks, [0, 6, 12, 18, 24, 30, 36, 42]);
}

#[test]
fn every_bar_starts_again_from_one() {
    let all = labels(TimeSignature::FOUR_FOUR, 2);
    assert_eq!(all.len(), 16);
    assert_eq!(all[8], Beat(1));
    assert_eq!(all[14], Beat(4));
}

#[test]
fn three_four_counts_three_beats() {
    let three_four = TimeSignature::new(3, 4).unwrap();
    assert_eq!(
        labels(three_four, 1),
        [Beat(1), And, Beat(2), And, Beat(3), And]
    );
}

#[test]
fn six_eight_counts_every_8th_as_a_beat() {
    let six_eight = TimeSignature::new(6, 8).unwrap();
    assert_eq!(
        labels(six_eight, 1),
        [Beat(1), Beat(2), Beat(3), Beat(4), Beat(5), Beat(6)]
    );
}
