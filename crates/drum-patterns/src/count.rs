//! The 8th count shown above the notes: `1 и 2 и 3 и 4 и` in 4/4. It
//! depends only on the bar, not on what is played.

use crate::pattern::Pattern;
use crate::time::NoteValue;

#[cfg(test)]
#[path = "count_tests.rs"]
mod tests;

/// What is said on one 8th.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CountLabel {
    /// A beat, numbered from 1 in each bar.
    Beat(u8),
    /// The 8th between beats - "и" ("and").
    And,
}

/// One label of the count and where it sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CountMark {
    /// Ticks from the start of the pattern.
    pub tick: u32,
    /// What is said there.
    pub label: CountLabel,
}

/// The count for every 8th of the pattern. An 8th on a beat says the
/// beat's number; one between beats says "и". In 6/8 every 8th is a beat.
pub fn count(pattern: &Pattern) -> Vec<CountMark> {
    let time = pattern.time();
    let per_beat = time.ticks_per_beat();
    let per_bar = time.ticks_per_bar();
    (0..pattern.len_ticks())
        .step_by(NoteValue::Eighth.ticks() as usize)
        .map(|tick| {
            let in_bar = tick % per_bar;
            let label = if in_bar.is_multiple_of(per_beat) {
                CountLabel::Beat((in_bar / per_beat + 1) as u8)
            } else {
                CountLabel::And
            };
            CountMark { tick, label }
        })
        .collect()
}
