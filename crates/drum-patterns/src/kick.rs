//! The kick part. Base kicks sit on 8ths the snare leaves free; extra
//! kicks sit on the 16ths between, harder with each Generate:
//!
//! | Level | Allowed |
//! |---|---|
//! | 1 | the `a` before a beat, not next to an accent |
//! | 2 | any weak 16th, not next to an accent |
//! | 3 | next to accents too |
//!
//! A kick never shares a tick with a snare hit of any kind.

use rand::Rng;
use rand::seq::IndexedRandom;

use crate::ladder;
use crate::pattern::{Hit, HitKind, Instrument, Pattern};
use crate::time::NoteValue;

#[cfg(test)]
#[path = "kick_tests.rs"]
mod tests;

/// Rewrites the base kicks: `count` kicks a bar on 8ths that have no
/// snare hit of any kind - the first always on the 1, the rest at random.
/// Each bar gets its own. Asking for
/// more than the free 8ths places one on each. Kicks between the 8ths -
/// the extra kicks - and other instruments are left alone.
pub fn base_kicks(pattern: &mut Pattern, count: u8, rng: &mut impl Rng) {
    let per_bar = pattern.time().ticks_per_bar();
    let eighth = NoteValue::Eighth.ticks();
    let on_eighth = move |tick: u32| (tick % per_bar).is_multiple_of(eighth);
    pattern.retain(|h| h.instrument != Instrument::Kick || !on_eighth(h.tick));

    for bar in 0..pattern.bars() as u32 {
        let start = bar * per_bar;
        let free: Vec<u32> = (start..start + per_bar)
            .step_by(eighth as usize)
            .filter(|&t| pattern.get(t, Instrument::Snare).is_none())
            .collect();
        let (one, rest): (Vec<u32>, Vec<u32>) = free.iter().partition(|&&t| t == start);
        let others = (count as usize).saturating_sub(one.len());
        let chosen = one
            .into_iter()
            .take(count as usize)
            .chain(rest.sample(rng, others).copied());
        for tick in chosen {
            let hit = Hit::new(tick, Instrument::Kick, HitKind::Normal);
            pattern.set(hit).expect("every tick is inside the pattern");
        }
    }
}

/// The hardest extra-kick level; higher levels count as this one.
pub const EXTRA_KICK_LEVELS: u8 = 3;

/// Rewrites the extra kicks: `count` a bar on the 16ths between the 8ths,
/// at difficulty `level` (1 to [`EXTRA_KICK_LEVELS`]; 0 counts as 1),
/// never on a snare hit of any kind. Each bar gets its own. Base kicks and
/// other instruments are left alone.
pub fn extra_kicks(pattern: &mut Pattern, count: u8, level: u8, rng: &mut impl Rng) {
    let time = pattern.time();
    let per_bar = time.ticks_per_bar();
    let per_beat = time.ticks_per_beat();
    let sixteenth = NoteValue::Sixteenth.ticks();
    let weak = move |tick: u32| !(tick % per_bar).is_multiple_of(2 * sixteenth);
    pattern.retain(|h| h.instrument != Instrument::Kick || !weak(h.tick));

    let level = level.clamp(1, EXTRA_KICK_LEVELS);
    let near = ladder::Neighbours::new(pattern);
    let snare: Vec<u32> = pattern.hits_of(Instrument::Snare).map(|h| h.tick).collect();
    let accents: Vec<u32> = pattern
        .hits_of(Instrument::Snare)
        .filter(|h| h.kind == HitKind::Accent)
        .map(|h| h.tick)
        .collect();

    let allowed_at = |level: u8, tick: u32| {
        let before_beat = (tick % per_bar) % per_beat == per_beat - sixteenth;
        match level {
            1 => before_beat && !near.any(tick, &accents),
            2 => !near.any(tick, &accents),
            _ => true,
        }
    };

    for bar in 0..pattern.bars() as u32 {
        let start = bar * per_bar;
        let free: Vec<u32> = (start..start + per_bar)
            .step_by(sixteenth as usize)
            .filter(|&t| weak(t) && !snare.contains(&t))
            .collect();
        let at = |level: u8| -> Vec<u32> {
            free.iter()
                .copied()
                .filter(|&t| allowed_at(level, t))
                .collect()
        };
        let easier = if level > 1 { at(level - 1) } else { Vec::new() };
        for tick in ladder::pick(&easier, &at(level), count as usize, rng) {
            let hit = Hit::new(tick, Instrument::Kick, HitKind::Normal);
            pattern.set(hit).expect("every tick is inside the pattern");
        }
    }
}
