//! The Sound menu's presets: a sound for each instrument in each style,
//! made from formulas so any of them may ship. Until each style is
//! designed (step 6c), every preset plays the placeholder kit's sound.

use drum_patterns::Instrument;

use crate::kit::Sound;
use crate::synth::placeholder_sound;

#[cfg(test)]
#[path = "presets_tests.rs"]
mod tests;

/// A style of kit, as the Sound menu lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Preset {
    /// An open acoustic kit.
    #[default]
    Rock,
    /// A 909-style drum machine.
    HipHop,
    /// An 808-style drum machine.
    Electro,
    /// Soft hits, little attack: brushes or rods.
    Jazz,
    /// Aggressive triggered drums, hard attack.
    Metal,
}

impl Preset {
    /// Every preset, in the menu's order.
    pub const ALL: [Self; 5] = [
        Self::Rock,
        Self::HipHop,
        Self::Electro,
        Self::Jazz,
        Self::Metal,
    ];
}

/// `preset`'s sound for `instrument`, made at `rate`. The same arguments
/// always give the same sound.
pub fn preset_sound(preset: Preset, instrument: Instrument, rate: u32) -> Sound {
    // Every style is the placeholder until step 6c designs it.
    let _ = preset;
    placeholder_sound(instrument, rate)
}
