use super::*;
use crate::pattern::{Hit, HitKind, Instrument};
use crate::time::TimeSignature;

use Syllable::{Ka, Ki, Ta};

fn pattern_with(bars: u8, hits: &[(u32, Instrument)]) -> Pattern {
    let mut p = Pattern::new(TimeSignature::FOUR_FOUR, bars).unwrap();
    for &(tick, instrument) in hits {
        p.set(Hit::new(tick, instrument, HitKind::Normal)).unwrap();
    }
    p
}

fn hats(ticks: impl IntoIterator<Item = u32>) -> Pattern {
    let hits: Vec<(u32, Instrument)> = ticks.into_iter().map(|t| (t, Instrument::HiHat)).collect();
    pattern_with(1, &hits)
}

fn syllables(p: &Pattern) -> Vec<Syllable> {
    konnakol(p).into_iter().map(|m| m.syllable).collect()
}

#[test]
fn nothing_where_there_is_no_note() {
    assert!(konnakol(&hats([])).is_empty());
    let ticks: Vec<u32> = konnakol(&hats([0, 12])).iter().map(|m| m.tick).collect();
    assert_eq!(ticks, [0, 12]);
}

#[test]
fn notes_on_one_tick_share_a_syllable() {
    let p = pattern_with(
        1,
        &[
            (0, Instrument::Kick),
            (0, Instrument::HiHat),
            (0, Instrument::Crash),
        ],
    );
    assert_eq!(
        konnakol(&p),
        [KonnakolMark {
            tick: 0,
            syllable: Ta
        }]
    );
}

#[test]
fn straight_16ths_alternate_ta_ka() {
    let s = syllables(&hats((0..48).step_by(3)));
    assert_eq!(s.len(), 16);
    assert_eq!(&s[..4], [Ta, Ka, Ta, Ka]);
    assert!(s.chunks(2).all(|pair| pair == [Ta, Ka]));
}

#[test]
fn eighths_are_all_ta() {
    assert!(
        syllables(&hats((0..48).step_by(6)))
            .iter()
            .all(|&s| s == Ta)
    );
}

#[test]
fn a_lone_16th_between_8ths_is_ka() {
    assert_eq!(syllables(&hats([3, 21])), [Ka, Ka]);
}

#[test]
fn dotted_8ths_take_the_16th_they_land_on() {
    assert_eq!(
        syllables(&hats((0..48).step_by(9))),
        [Ta, Ka, Ta, Ka, Ta, Ka]
    );
}

#[test]
fn eighth_triplets_are_ta_ki_ta() {
    let s = syllables(&hats((0..48).step_by(4)));
    assert_eq!(s.len(), 12);
    assert!(s.chunks(3).all(|group| group == [Ta, Ki, Ta]));
}

#[test]
fn sixteenth_triplets_are_ta_ki_ta_twice_a_beat() {
    let s = syllables(&hats((0..48).step_by(2)));
    assert_eq!(s.len(), 24);
    assert!(s.chunks(3).all(|group| group == [Ta, Ki, Ta]));
}

#[test]
fn quarter_triplets_group_across_two_beats() {
    assert_eq!(
        syllables(&hats((0..48).step_by(8))),
        [Ta, Ki, Ta, Ta, Ki, Ta]
    );
}

#[test]
fn a_backbeat_does_not_break_quarter_triplets() {
    let mut hits: Vec<(u32, Instrument)> =
        (0..48).step_by(8).map(|t| (t, Instrument::HiHat)).collect();
    hits.extend([(12, Instrument::Snare), (36, Instrument::Snare)]);
    let p = pattern_with(1, &hits);

    let at = |tick: u32| {
        konnakol(&p)
            .into_iter()
            .find(|m| m.tick == tick)
            .unwrap()
            .syllable
    };
    assert_eq!([at(0), at(8), at(16)], [Ta, Ki, Ta]);
    assert_eq!(at(12), Ta);
}

#[test]
fn a_lone_8th_triplet_middle_is_ki() {
    assert_eq!(syllables(&hats([4])), [Ki]);
}

#[test]
fn each_beat_reads_its_own_triplets() {
    // Beat 1: 8th triplets. Beat 2: 16th triplets.
    let s = syllables(&hats([0, 4, 8, 12, 14, 16, 18, 20, 22]));
    assert_eq!(s, [Ta, Ki, Ta, Ta, Ki, Ta, Ta, Ki, Ta]);
}

#[test]
fn every_bar_reads_the_same() {
    let one: Vec<(u32, Instrument)> = [0, 3, 4, 8, 15].map(|t| (t, Instrument::Snare)).to_vec();
    let two: Vec<(u32, Instrument)> = one
        .iter()
        .flat_map(|&(t, i)| [(t, i), (t + 48, i)])
        .collect();
    let first = syllables(&pattern_with(1, &one));
    let both = syllables(&pattern_with(2, &two));
    assert_eq!(both[..first.len()], first);
    assert_eq!(both[first.len()..], first);
}

#[test]
fn quarter_triplets_need_a_whole_span_in_three_four() {
    // 3/4's last beat has no partner, so 8 ticks into it reads as an 8th triplet.
    let mut p = Pattern::new(TimeSignature::new(3, 4).unwrap(), 1).unwrap();
    p.set(Hit::new(28, Instrument::HiHat, HitKind::Normal))
        .unwrap();
    assert_eq!(syllables(&p), [Ki]);
}
