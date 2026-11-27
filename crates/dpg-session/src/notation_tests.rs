use drum_patterns::{Hit, HitKind, Instrument, NoteValue, TimeSignature, hi_hat_pulse};

use super::*;

fn bar(ticks: &[u32]) -> Pattern {
    let mut p = Pattern::new(TimeSignature::FOUR_FOUR, 1).unwrap();
    for &t in ticks {
        p.set(Hit::new(t, Instrument::HiHat, HitKind::Normal))
            .unwrap();
    }
    p
}

fn pulse(value: NoteValue) -> Pattern {
    let mut p = bar(&[]);
    hi_hat_pulse(&mut p, value, 0);
    p
}

/// Each group of the first bar as `8 16r 8.`, groups split by `|`.
fn rhythm(pattern: &Pattern) -> String {
    notation(pattern).bars[0]
        .iter()
        .map(|g| {
            g.events
                .iter()
                .map(|e| {
                    let dots = ".".repeat(e.dots as usize);
                    let rest = if e.rest { "r" } else { "" };
                    format!("{}{dots}{rest}", e.value)
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

fn tuplets(pattern: &Pattern) -> Vec<Option<Tuplet>> {
    notation(pattern).bars[0].iter().map(|g| g.tuplet).collect()
}

fn labels(pattern: &Pattern) -> Vec<String> {
    notation(pattern).bars[0]
        .iter()
        .flat_map(|g| &g.events)
        .filter_map(|e| e.label.clone())
        .collect()
}

#[test]
fn straight_pulses_fill_each_beat() {
    assert_eq!(rhythm(&pulse(NoteValue::Quarter)), "4 | 4 | 4 | 4");
    assert_eq!(rhythm(&pulse(NoteValue::Eighth)), "8 8 | 8 8 | 8 8 | 8 8");
    assert_eq!(
        rhythm(&pulse(NoteValue::Sixteenth)),
        "16 16 16 16 | 16 16 16 16 | 16 16 16 16 | 16 16 16 16"
    );
    assert_eq!(tuplets(&pulse(NoteValue::Eighth)), [None; 4]);
}

#[test]
fn an_empty_beat_is_a_quarter_rest() {
    assert_eq!(rhythm(&bar(&[])), "4r | 4r | 4r | 4r");
}

#[test]
fn a_note_lasts_to_the_next_inside_its_beat() {
    // 1, 1e; then the a of 2 alone; then the e of 3 alone.
    assert_eq!(
        rhythm(&bar(&[0, 3, 21, 27])),
        "16 8. | 8r 16r 16 | 16r 8. | 4r"
    );
}

#[test]
fn rests_split_where_the_beat_divides() {
    // A note on the a: three 16ths of rest before it, as 8th + 16th.
    assert!(rhythm(&bar(&[9])).starts_with("8r 16r 16 |"));
}

#[test]
fn triplets_get_their_bracket() {
    assert_eq!(
        rhythm(&pulse(NoteValue::EighthTriplet)),
        "8 8 8 | 8 8 8 | 8 8 8 | 8 8 8"
    );
    assert_eq!(tuplets(&pulse(NoteValue::EighthTriplet)), [tuplet(3, 2); 4]);
    assert_eq!(
        rhythm(&pulse(NoteValue::SixteenthTriplet))
            .split(" | ")
            .next(),
        Some("16 16 16 16 16 16")
    );
    assert_eq!(
        tuplets(&pulse(NoteValue::SixteenthTriplet))[0],
        tuplet(6, 4)
    );
}

#[test]
fn quarter_triplets_take_two_beats_under_one_bracket() {
    let p = pulse(NoteValue::QuarterTriplet);
    assert_eq!(rhythm(&p), "4 4 4 | 4 4 4");
    assert_eq!(tuplets(&p), [tuplet(3, 2); 2]);
    assert_eq!(notation(&p).bars[0][1].tick, 24);
}

#[test]
fn quarter_triplets_with_a_straight_note_go_beat_by_beat() {
    let mut p = pulse(NoteValue::QuarterTriplet);
    p.set(Hit::new(12, Instrument::Snare, HitKind::Normal))
        .unwrap();
    assert_eq!(
        rhythm(&p).split(" | ").take(2).collect::<Vec<_>>(),
        ["4 8", "8 4"]
    );
    assert_eq!(tuplets(&p)[..2], [tuplet(3, 2); 2]);
}

#[test]
fn a_dotted_pulse_is_written_inside_each_beat() {
    assert_eq!(
        rhythm(&pulse(NoteValue::DottedEighth)),
        "8. 16 | 8r 8 | 16r 8. | 8. 16"
    );
    assert_eq!(
        rhythm(&pulse(NoteValue::DottedQuarter)),
        "4 | 8r 8 | 4r | 4"
    );
}

#[test]
fn triplets_and_straight_8ths_in_one_beat_share_the_16th_triplet_grid() {
    // 8th triplets with an accent on the и.
    let mut p = bar(&[0, 4, 8]);
    p.set(Hit::new(6, Instrument::Snare, HitKind::Accent))
        .unwrap();
    assert!(rhythm(&p).starts_with("8 16 16 8 |"));
    assert_eq!(tuplets(&p)[0], tuplet(6, 4));
}

#[test]
fn sixteenths_and_8th_triplets_in_one_beat_still_spell() {
    // 1, its e and the first triplet's second note: twelve 32nds a beat.
    let p = bar(&[0, 3, 4]);
    assert!(rhythm(&p).starts_with("16. 32 4 |"));
    assert_eq!(tuplets(&p)[0], tuplet(12, 8));
}

#[test]
fn labels_name_each_note_value() {
    assert_eq!(labels(&bar(&[0, 3])), ["16", "8."]);
    assert_eq!(labels(&bar(&[0, 4, 8]))[..3], ["8t", "8t", "8t"]);
    assert_eq!(
        labels(&pulse(NoteValue::QuarterTriplet))[..3],
        ["4t", "4t", "4t"]
    );
}

#[test]
fn a_note_holds_every_instrument_struck_with_it() {
    let mut p = bar(&[0]);
    p.set(Hit::new(0, Instrument::Kick, HitKind::Accent))
        .unwrap();
    let first = &notation(&p).bars[0][0].events[0];
    assert_eq!(
        first.hits,
        [
            NoteHit {
                instrument: InstrumentName::HiHat,
                kind: KindName::Normal
            },
            NoteHit {
                instrument: InstrumentName::Kick,
                kind: KindName::Accent
            },
        ]
    );
}

#[test]
fn konnakol_sits_under_notes_only() {
    let n = notation(&bar(&[0, 3, 6]));
    let syllables: Vec<_> = n.bars[0]
        .iter()
        .flat_map(|g| &g.events)
        .map(|e| e.konnakol)
        .collect();
    assert_eq!(syllables[..4], [Some("т"), Some("к"), Some("т"), None]);
}

#[test]
fn the_count_says_each_8th() {
    let labels: Vec<_> = notation(&bar(&[]))
        .count
        .into_iter()
        .map(|c| c.label)
        .collect();
    assert_eq!(labels, ["1", "и", "2", "и", "3", "и", "4", "и"]);
}

#[test]
fn every_bar_is_spelled() {
    let p = Pattern::new(TimeSignature::FOUR_FOUR, 2).unwrap();
    let n = notation(&p);
    assert_eq!(n.bars.len(), 2);
    assert_eq!(n.bars[1][0].tick, 48);
}

#[test]
fn groups_fill_their_time_exactly() {
    for value in NoteValue::ALL {
        for group in &notation(&pulse(value)).bars[0] {
            let sum: u32 = group.events.iter().map(|e| e.ticks).sum();
            assert_eq!(sum, group.ticks, "{value:?}");
        }
    }
}

#[test]
fn any_groove_spells_every_note_once_and_fills_its_time() {
    use drum_patterns::Groove;
    for seed in 0..40 {
        for value in NoteValue::ALL {
            let groove = Groove {
                bars: 2,
                hi_hat: value,
                hi_hat_random: (seed % 2 == 0).then_some(50),
                accents: 2,
                ghosts: 3,
                ghost_level: 4,
                extra_kicks: 2,
                extra_kick_level: 3,
                seed,
                ..Groove::default()
            };
            for lap in groove.build_laps().unwrap() {
                let n = notation(&lap);
                let groups = n.bars.iter().flatten();
                let mut written = Vec::new();
                for group in groups {
                    let sum: u32 = group.events.iter().map(|e| e.ticks).sum();
                    assert_eq!(sum, group.ticks, "{seed} {value:?}");
                    let notes = group.events.iter().filter(|e| !e.rest);
                    written.extend(notes.map(|e| e.tick));
                }
                let mut struck: Vec<u32> = lap.hits().iter().map(|h| h.tick).collect();
                struck.dedup();
                assert_eq!(written, struck, "{seed} {value:?}");
            }
        }
    }
}
