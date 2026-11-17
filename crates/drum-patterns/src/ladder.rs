//! The difficulty ladder behind Generate: each level opens positions the
//! level below kept closed, and a variant at that level uses at least one
//! of them - so pressing again really is harder, not just different.

use rand::Rng;
use rand::seq::IndexedRandom;

use crate::pattern::Pattern;
use crate::time::NoteValue;

#[cfg(test)]
#[path = "ladder_tests.rs"]
mod tests;

/// Picks `count` of the `allowed` ticks at random, at least one of them -
/// when there is one - from those the `easier` level didn't allow.
/// Returns them in tick order; fewer when fewer are allowed.
pub(crate) fn pick(easier: &[u32], allowed: &[u32], count: usize, rng: &mut impl Rng) -> Vec<u32> {
    let fresh: Vec<u32> = allowed
        .iter()
        .copied()
        .filter(|t| !easier.contains(t))
        .collect();
    let mut chosen: Vec<u32> = Vec::with_capacity(count);
    if count > 0
        && let Some(&first) = fresh.choose(rng)
    {
        chosen.push(first);
    }
    let rest: Vec<u32> = allowed
        .iter()
        .copied()
        .filter(|t| !chosen.contains(t))
        .collect();
    chosen.extend(rest.sample(rng, count - chosen.len()).copied());
    chosen.sort_unstable();
    chosen
}

/// Whether a tick sits a 16th away from others, counting across the loop
/// point - the pattern repeats, so its last 16th is next to its first.
pub(crate) struct Neighbours {
    len: u32,
}

impl Neighbours {
    pub(crate) fn new(pattern: &Pattern) -> Self {
        Self {
            len: pattern.len_ticks(),
        }
    }

    pub(crate) fn any(&self, tick: u32, others: &[u32]) -> bool {
        let step = NoteValue::Sixteenth.ticks();
        let before = (tick + self.len - step) % self.len;
        let after = (tick + step) % self.len;
        others.iter().any(|&o| o == before || o == after)
    }
}
