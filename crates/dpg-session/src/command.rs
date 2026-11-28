//! Every button and fader on the screen, as one command each. The UI sends
//! them as JSON: `{"type": "pulse", "value": "8."}`.

use serde::Deserialize;

use crate::names::{
    FeelName, HiHatAccentsName, InstrumentName, PresetName, PulseName, SubdivisionName,
};
use crate::session::View;

#[cfg(test)]
#[path = "command_tests.rs"]
mod tests;

/// A fader on the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Fader {
    /// Over every drum.
    Drums,
    /// The metronome's clicks.
    Metronome,
    /// One instrument's own fader.
    #[serde(untagged)]
    Instrument(InstrumentName),
}

/// What the user did.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Command {
    /// Start, or Stop while playing.
    StartStop,
    /// Sets the tempo.
    Bpm {
        /// Beats a minute, kept to 30-300.
        bpm: f64,
    },
    /// BPM - or +.
    NudgeBpm {
        /// Beats a minute to add; negative to take away.
        by: f64,
    },
    /// Tap tempo.
    Tap,
    /// Turns the metronome on or off.
    Metronome,
    /// The metronome's clicks a beat.
    Subdivision {
        /// `4`, `8`, `8t` or `16`.
        value: SubdivisionName,
    },
    /// The metronome's next sound; after the last, the first.
    MetronomeSound,
    /// An instrument's kit style, from the Sound menu. The crash has no
    /// choice of its own: it follows the hi-hat.
    SoundPreset {
        /// The hi-hat, snare or kick.
        instrument: InstrumentName,
        /// The style.
        preset: PresetName,
    },
    /// The pattern's length.
    Bars {
        /// 1 to 4.
        count: u8,
    },
    /// Stacked or scrolled bars.
    View {
        /// How the bars are laid out.
        view: View,
    },
    /// Bar skip on, playing `play` bars and silencing `skip`.
    BarSkip {
        /// Bars heard.
        play: u8,
        /// Bars silent.
        skip: u8,
    },
    /// Bar skip off.
    BarSkipOff,
    /// Where the crash plays.
    Crash {
        /// Every this many bars; `None` is once at the start.
        every: Option<u8>,
    },
    /// The hi-hat pulse, played in full.
    Pulse {
        /// `4`, `8`, `16`, `4t`, `8t`, `16t`, `4.` or `8.`.
        value: PulseName,
    },
    /// The hi-hat's Random.
    HiHatRandom,
    /// How many variants the hi-hat's Random holds.
    HiHatRandomCount {
        /// 1 to 24.
        count: u8,
    },
    /// The hi-hat's Density.
    Density {
        /// Off-beat hits kept, in percent.
        percent: u8,
    },
    /// Where the hi-hat is accented.
    HiHatAccents {
        /// Off, on the beats or on the и.
        accents: HiHatAccentsName,
    },
    /// The snare's time feel.
    Feel {
        /// `basic`, `half` or `double`.
        feel: FeelName,
    },
    /// Snare accents a bar.
    Accents {
        /// Accents a bar.
        count: u8,
    },
    /// Ghost notes a bar.
    Ghosts {
        /// Ghost notes a bar.
        count: u8,
    },
    /// Ghost notes' Generate.
    GenerateGhosts,
    /// Ghost notes' Random.
    GhostRandom,
    /// How many variants the ghost notes' Random holds.
    GhostRandomCount {
        /// 1 to 24.
        count: u8,
    },
    /// Base kicks a bar.
    BaseKicks {
        /// Kicks a bar.
        count: u8,
    },
    /// Base kicks' Generate.
    GenerateBaseKicks,
    /// Extra kicks a bar.
    ExtraKicks {
        /// Kicks a bar.
        count: u8,
    },
    /// Extra kicks' Generate.
    GenerateExtraKicks,
    /// Moves a fader.
    Fader {
        /// Which fader.
        fader: Fader,
        /// 0 to 1.
        position: f32,
    },
    /// Turns an instrument's solo on or off.
    Solo {
        /// The instrument.
        instrument: InstrumentName,
    },
    /// Turns an instrument's mute on or off.
    Mute {
        /// The instrument.
        instrument: InstrumentName,
    },
}
