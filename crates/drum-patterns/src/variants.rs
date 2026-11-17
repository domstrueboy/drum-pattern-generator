//! Random: a set of N variants that each press steps through. A set is
//! fixed by its seed, so the app keeps only the seed, N and where it is,
//! and any variant can be built again.

use rand::SeedableRng;

#[cfg(test)]
#[path = "variants_tests.rs"]
mod tests;

/// A set of 1 to [`Variants::MAX`] variants. Variant `k` is whatever a
/// generator writes with [`Variants::rng`]`(k)` - the hi-hat for the
/// hi-hat's Random, the ghost notes for theirs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Variants {
    seed: u64,
    count: u8,
}

impl Variants {
    /// The most variants a set holds.
    pub const MAX: u8 = 24;

    /// A set of `count` variants, clamped to 1 to [`Self::MAX`].
    pub fn new(seed: u64, count: u8) -> Self {
        Self {
            seed,
            count: count.clamp(1, Self::MAX),
        }
    }

    /// How many variants the set holds.
    pub fn count(&self) -> u8 {
        self.count
    }

    /// The variant after `index` - the first again after the last.
    pub fn next(&self, index: u8) -> u8 {
        (index % self.count + 1) % self.count
    }

    /// The random source for variant `index` (taken modulo the count):
    /// always the same for the same set and index, different between
    /// variants.
    pub fn rng<R: SeedableRng>(&self, index: u8) -> R {
        const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15; // spreads nearby indices apart
        let index = u64::from(index % self.count);
        R::seed_from_u64(self.seed ^ (index + 1).wrapping_mul(GOLDEN))
    }
}
