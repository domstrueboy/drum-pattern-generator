//! Ghost notes: quiet snare hits on the 16ths, simple at first and harder
//! with each Generate.
//!
//! | Level | Allowed |
//! |---|---|
//! | 1 | weak 16ths, not next to an accent, not on a kick |
//! | 2 | weak 16ths, next to accents too |
//! | 3 | any free 16th, 8ths too; still not on a kick |
//! | 4 | any free 16th, on a kick too |

use rand::Rng;

use crate::ladder;
use crate::pattern::{Hit, HitKind, Instrument, Pattern};
use crate::time::NoteValue;

#[cfg(test)]
#[path = "ghost_notes_tests.rs"]
mod tests;

/// The hardest ghost-note level; higher levels count as this one.
pub const GHOST_LEVELS: u8 = 4;

/// Rewrites the ghost notes: `count` a bar at difficulty `level` (1 to
/// [`GHOST_LEVELS`]; 0 counts as 1), never on another snare hit. Each bar
/// gets its own. Main hits, accents and other instruments are left alone.
pub fn ghost_notes(pattern: &mut Pattern, count: u8, level: u8, rng: &mut impl Rng) {
    pattern.retain(|h| h.instrument != Instrument::Snare || h.kind != HitKind::Ghost);
    let level = level.clamp(1, GHOST_LEVELS);
    let near = ladder::Neighbours::new(pattern);
    let snare: Vec<u32> = pattern.hits_of(Instrument::Snare).map(|h| h.tick).collect();
    let accents: Vec<u32> = pattern
        .hits_of(Instrument::Snare)
        .filter(|h| h.kind == HitKind::Accent)
        .map(|h| h.tick)
        .collect();
    let kicks: Vec<u32> = pattern.hits_of(Instrument::Kick).map(|h| h.tick).collect();
    let per_bar = pattern.time().ticks_per_bar();
    let sixteenth = NoteValue::Sixteenth.ticks();

    let allowed_at = |level: u8, tick: u32| {
        let weak = !(tick % per_bar).is_multiple_of(2 * sixteenth);
        let on_kick = kicks.contains(&tick);
        match level {
            1 => weak && !near.any(tick, &accents) && !on_kick,
            2 => weak && !on_kick,
            3 => !on_kick,
            _ => true,
        }
    };

    for bar in 0..pattern.bars() as u32 {
        let start = bar * per_bar;
        let free: Vec<u32> = (start..start + per_bar)
            .step_by(sixteenth as usize)
            .filter(|t| !snare.contains(t))
            .collect();
        let at = |level: u8| -> Vec<u32> {
            free.iter()
                .copied()
                .filter(|&t| allowed_at(level, t))
                .collect()
        };
        let easier = if level > 1 { at(level - 1) } else { Vec::new() };
        for tick in ladder::pick(&easier, &at(level), count as usize, rng) {
            let hit = Hit::new(tick, Instrument::Snare, HitKind::Ghost);
            pattern.set(hit).expect("every tick is inside the pattern");
        }
    }
}
