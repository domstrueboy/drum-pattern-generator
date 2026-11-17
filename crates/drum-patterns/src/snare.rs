//! The snare part: the feel's main hits, then accents on 8ths on top.
//! Ghost notes come later (step 2f).

use rand::Rng;
use rand::seq::IndexedRandom;

use crate::pattern::{Hit, HitKind, Instrument, Pattern};
use crate::time::NoteValue;

#[cfg(test)]
#[path = "snare_tests.rs"]
mod tests;

/// Where the snare's main hits sit. Written for 4/4; in other time
/// signatures each takes the same shape over the bar's beats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Feel {
    /// The backbeat: beats 2 and 4 - every even beat.
    #[default]
    Basic,
    /// Beat 3 only - the beat halfway through the bar.
    HalfTime,
    /// Every "и" - 2 and 4 counted at double speed, four a bar.
    DoubleTime,
}

impl Feel {
    /// The main hits' ticks within one bar of `beats` beats of
    /// `per_beat` ticks.
    fn ticks_in_bar(self, beats: u32, per_beat: u32) -> Vec<u32> {
        match self {
            Self::Basic => (1..beats).step_by(2).map(|b| b * per_beat).collect(),
            Self::HalfTime => vec![beats / 2 * per_beat],
            Self::DoubleTime => (0..beats).map(|b| b * per_beat + per_beat / 2).collect(),
        }
    }
}

/// Rewrites the snare: the feel's main hits, then `accents` accented hits
/// a bar, placed at random on 8ths free of the main hits and of any kick
/// already written. Each bar gets its own accents - a 1-bar pattern
/// repeats them. Asking for more accents than free 8ths places one on
/// each. Other instruments are left alone.
pub fn snare(pattern: &mut Pattern, feel: Feel, accents: u8, rng: &mut impl Rng) {
    pattern.clear(Instrument::Snare);
    let time = pattern.time();
    let per_bar = time.ticks_per_bar();
    let main = feel.ticks_in_bar(time.beats() as u32, time.ticks_per_beat());
    let eighths: Vec<u32> = (0..per_bar)
        .step_by(NoteValue::Eighth.ticks() as usize)
        .filter(|t| !main.contains(t))
        .collect();

    for bar in 0..pattern.bars() as u32 {
        let start = bar * per_bar;
        let free: Vec<u32> = eighths
            .iter()
            .copied()
            .filter(|&t| pattern.get(start + t, Instrument::Kick).is_none())
            .collect();
        let accented: Vec<u32> = free.sample(rng, accents as usize).copied().collect();
        let hits = main
            .iter()
            .map(|&t| (t, HitKind::Normal))
            .chain(accented.into_iter().map(|t| (t, HitKind::Accent)));
        for (tick, kind) in hits {
            let hit = Hit::new(start + tick, Instrument::Snare, kind);
            pattern.set(hit).expect("every tick is inside the pattern");
        }
    }
}
