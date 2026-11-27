//! The pattern spelled as notation: each beat on the simplest grid that
//! holds its notes - 16ths, 8th triplets, 16th triplets - with note
//! values, dots, rests and tuplet brackets, the 8th count above and
//! konnakol under the notes. The UI only draws what this decides.

use std::collections::BTreeMap;

use drum_patterns::{CountLabel, Pattern, Syllable, count, konnakol};
use serde::Serialize;

use crate::names::{InstrumentName, KindName};

#[cfg(test)]
#[path = "notation_tests.rs"]
mod tests;

/// One pass through the pattern, ready to draw.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Notation {
    /// The 8th count, `1 и 2 и ...`, over the whole pattern.
    pub count: Vec<CountView>,
    /// Each bar's groups, in order.
    pub bars: Vec<Vec<Group>>,
}

/// One label of the count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CountView {
    /// From the pattern's start.
    pub tick: u32,
    /// A beat's number, or `и`.
    pub label: String,
}

/// A beat - or two, under one quarter-triplet bracket - beamed together.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Group {
    /// Where it starts, from the pattern's start.
    pub tick: u32,
    /// How long it is.
    pub ticks: u32,
    /// Its bracket, when its notes are triplets.
    pub tuplet: Option<Tuplet>,
    /// Its notes and rests, in order.
    pub events: Vec<Event>,
}

/// A tuplet bracket: `notes` in the time of `occupied`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Tuplet {
    /// Notes written - 3 for a triplet.
    pub notes: u8,
    /// Notes of that value the time normally holds - 2 for a triplet.
    pub occupied: u8,
}

/// A note or a rest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Event {
    /// Where it sits, from the pattern's start.
    pub tick: u32,
    /// How long it lasts.
    pub ticks: u32,
    /// The written value: 4 a quarter, 8 an 8th, 16 a 16th...
    pub value: u8,
    /// Dots after it.
    pub dots: u8,
    /// A rest rather than a note.
    pub rest: bool,
    /// The note-value row's label - `16`, `8.`, `8t` - on notes only.
    pub label: Option<String>,
    /// What is struck; empty for a rest.
    pub hits: Vec<NoteHit>,
    /// The konnakol syllable's letter, on notes only.
    pub konnakol: Option<&'static str>,
}

/// One instrument in a note.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct NoteHit {
    /// What is struck.
    pub instrument: InstrumentName,
    /// How hard.
    pub kind: KindName,
}

/// The grid a group is written on: its step in ticks, the written value of
/// one step, and the bracket that takes it there.
#[derive(Clone, Copy)]
struct Grid {
    step: u32,
    value: u8,
    tuplet: Option<Tuplet>,
}

const fn tuplet(notes: u8, occupied: u8) -> Option<Tuplet> {
    Some(Tuplet { notes, occupied })
}

/// Spells `pattern`.
pub fn notation(pattern: &Pattern) -> Notation {
    let time = pattern.time();
    let (per_beat, per_bar) = (time.ticks_per_beat(), time.ticks_per_bar());
    let mut notes: BTreeMap<u32, Vec<NoteHit>> = BTreeMap::new();
    for hit in pattern.hits() {
        notes.entry(hit.tick).or_default().push(NoteHit {
            instrument: hit.instrument.into(),
            kind: hit.kind.into(),
        });
    }
    let syllables: BTreeMap<u32, &'static str> = konnakol(pattern)
        .into_iter()
        .map(|m| (m.tick, letter(m.syllable)))
        .collect();
    let speller = Speller {
        notes: &notes,
        syllables: &syllables,
    };

    let beats = u32::from(time.beats());
    let bars = (0..u32::from(pattern.bars()))
        .map(|bar| {
            let mut groups = Vec::new();
            let mut beat = 0;
            while beat < beats {
                let start = bar * per_bar + beat * per_beat;
                // Two beats whose notes fall only on the quarter-triplet
                // grid read as one bracket of three quarters.
                let pair = 2 * per_beat;
                if per_beat == 12 && beat + 2 <= beats && speller.step(start, pair) == Some(8) {
                    groups.push(speller.group(start, pair, QUARTER_TRIPLETS));
                    beat += 2;
                    continue;
                }
                let grid = match speller.step(start, per_beat) {
                    Some(step) if step.is_multiple_of(3) => SIXTEENTHS,
                    Some(4) => EIGHTH_TRIPLETS,
                    Some(2) => SIXTEENTH_TRIPLETS,
                    Some(_) => THIRTY_SECOND_SEXTUPLETS,
                    None => SIXTEENTHS,
                };
                groups.push(speller.group(start, per_beat, grid));
                beat += 1;
            }
            groups
        })
        .collect();

    Notation {
        count: count(pattern)
            .into_iter()
            .map(|m| CountView {
                tick: m.tick,
                label: match m.label {
                    CountLabel::Beat(n) => n.to_string(),
                    CountLabel::And => "и".into(),
                },
            })
            .collect(),
        bars,
    }
}

const SIXTEENTHS: Grid = Grid {
    step: 3,
    value: 16,
    tuplet: None,
};
const EIGHTH_TRIPLETS: Grid = Grid {
    step: 4,
    value: 8,
    tuplet: tuplet(3, 2),
};
const SIXTEENTH_TRIPLETS: Grid = Grid {
    step: 2,
    value: 16,
    tuplet: tuplet(6, 4),
};
/// Straight 16ths and 8th triplets in one beat: twelve 32nds in the time
/// of eight. Rare, and the one grid that holds both.
const THIRTY_SECOND_SEXTUPLETS: Grid = Grid {
    step: 1,
    value: 32,
    tuplet: tuplet(12, 8),
};
const QUARTER_TRIPLETS: Grid = Grid {
    step: 8,
    value: 4,
    tuplet: tuplet(3, 2),
};

struct Speller<'a> {
    notes: &'a BTreeMap<u32, Vec<NoteHit>>,
    syllables: &'a BTreeMap<u32, &'static str>,
}

impl Speller<'_> {
    /// The coarsest step that holds every note in `start..start + len`
    /// and divides `len`; `None` when there is no note.
    fn step(&self, start: u32, len: u32) -> Option<u32> {
        let mut onsets = self.notes.range(start..start + len).map(|(t, _)| t - start);
        let first = onsets.next()?;
        Some(onsets.fold(gcd(len, first), gcd))
    }

    fn group(&self, start: u32, len: u32, grid: Grid) -> Group {
        let units = len / grid.step;
        let onsets: Vec<u32> = self
            .notes
            .range(start..start + len)
            .map(|(t, _)| (t - start) / grid.step)
            .collect();
        let mut events = Vec::new();
        let fill = |events: &mut Vec<Event>, from: u32, to: u32| {
            for (at, size) in rests(from, to) {
                events.push(self.event(start, grid, at, size, true));
            }
        };
        fill(&mut events, 0, onsets.first().copied().unwrap_or(units));
        for (i, &at) in onsets.iter().enumerate() {
            let next = onsets.get(i + 1).copied().unwrap_or(units);
            let size = note_size(next - at);
            events.push(self.event(start, grid, at, size, false));
            fill(&mut events, at + size, next);
        }
        Group {
            tick: start,
            ticks: len,
            tuplet: grid.tuplet,
            events,
        }
    }

    fn event(&self, start: u32, grid: Grid, at: u32, size: u32, rest: bool) -> Event {
        let tick = start + at * grid.step;
        let (value, dots) = written(grid.value, size);
        let label = (!rest).then(|| {
            let dot = if dots > 0 { "." } else { "" };
            let triplet = if grid.tuplet.is_some() { "t" } else { "" };
            format!("{value}{dot}{triplet}")
        });
        Event {
            tick,
            ticks: size * grid.step,
            value,
            dots,
            rest,
            label,
            hits: if rest {
                Vec::new()
            } else {
                self.notes[&tick].clone()
            },
            konnakol: if rest {
                None
            } else {
                self.syllables.get(&tick).copied()
            },
        }
    }
}

/// The longest single note, plain or dotted, that fits `units`: a power
/// of two, or three times one.
fn note_size(units: u32) -> u32 {
    let plain = 1 << units.ilog2();
    let dotted = plain / 2 * 3;
    if dotted > plain && dotted <= units {
        dotted
    } else {
        plain
    }
}

/// Rests from `from` to `to`, each as long as it can be while starting on
/// a multiple of its own length - so a rest never hides where the beat
/// divides.
fn rests(mut from: u32, to: u32) -> Vec<(u32, u32)> {
    let mut out = Vec::new();
    while from < to {
        let mut size = 1 << (to - from).ilog2();
        while !from.is_multiple_of(size) {
            size /= 2;
        }
        out.push((from, size));
        from += size;
    }
    out
}

/// `units` steps of `value` as one written value and its dots.
fn written(value: u8, units: u32) -> (u8, u8) {
    if units.is_power_of_two() {
        (value / units as u8, 0)
    } else {
        (value / (units / 3 * 2) as u8, 1)
    }
}

fn letter(syllable: Syllable) -> &'static str {
    match syllable {
        Syllable::Ta => "т",
        Syllable::Ka | Syllable::Ki => "к",
    }
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}
