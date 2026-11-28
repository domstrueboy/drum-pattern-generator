//! The model's choices under the names the screen uses - `8.`, `8t`,
//! `half` - so commands and snapshots read like the spec's menus.

use dpg_engine::{ClickSound, Subdivision};
use dpg_samples::Preset;
use drum_patterns::{Feel, HiHatAccents, HitKind, Instrument, NoteValue};
use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "names_tests.rs"]
mod tests;

/// A name enum, one variant for each model value, converting both ways.
macro_rules! names {
    ($(#[$meta:meta])* $name:ident => $model:ty { $($variant:ident = $label:literal => $value:path),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum $name {
            $(
                #[doc = concat!("`", $label, "`")]
                #[serde(rename = $label)]
                $variant,
            )*
        }

        impl From<$name> for $model {
            fn from(name: $name) -> Self {
                match name {
                    $($name::$variant => $value,)*
                }
            }
        }

        impl From<$model> for $name {
            fn from(value: $model) -> Self {
                match value {
                    $($value => $name::$variant,)*
                }
            }
        }
    };
}

names! {
    /// A hi-hat pulse, as the Hi-hat submenu writes it.
    PulseName => NoteValue {
        Quarter = "4" => NoteValue::Quarter,
        Eighth = "8" => NoteValue::Eighth,
        Sixteenth = "16" => NoteValue::Sixteenth,
        QuarterTriplet = "4t" => NoteValue::QuarterTriplet,
        EighthTriplet = "8t" => NoteValue::EighthTriplet,
        SixteenthTriplet = "16t" => NoteValue::SixteenthTriplet,
        DottedQuarter = "4." => NoteValue::DottedQuarter,
        DottedEighth = "8." => NoteValue::DottedEighth,
    }
}

names! {
    /// The metronome's clicks a beat, as its buttons write them.
    SubdivisionName => Subdivision {
        Quarter = "4" => Subdivision::Quarter,
        Eighth = "8" => Subdivision::Eighth,
        Triplet = "8t" => Subdivision::Triplet,
        Sixteenth = "16" => Subdivision::Sixteenth,
    }
}

names! {
    /// Where the hi-hat is accented.
    HiHatAccentsName => HiHatAccents {
        Off = "off" => HiHatAccents::Off,
        Beats = "beats" => HiHatAccents::Beats,
        Offbeats = "offbeats" => HiHatAccents::Offbeats,
    }
}

names! {
    /// The metronome's sound, as the Sound button steps through them.
    ClickSoundName => ClickSound {
        Click = "click" => ClickSound::Click,
        Ableton = "ableton" => ClickSound::Ableton,
        Cowbell = "cowbell" => ClickSound::Cowbell,
        Clave = "clave" => ClickSound::Clave,
        Bongo = "bongo" => ClickSound::Bongo,
        Electronic = "electronic" => ClickSound::Electronic,
    }
}

names! {
    /// A kit style in the Sound menu.
    PresetName => Preset {
        Rock = "rock" => Preset::Rock,
        HipHop = "hipHop" => Preset::HipHop,
        Electro = "electro" => Preset::Electro,
        Jazz = "jazz" => Preset::Jazz,
        Metal = "metal" => Preset::Metal,
    }
}

names! {
    /// The snare's time feel.
    FeelName => Feel {
        Basic = "basic" => Feel::Basic,
        HalfTime = "half" => Feel::HalfTime,
        DoubleTime = "double" => Feel::DoubleTime,
    }
}

names! {
    /// An instrument.
    InstrumentName => Instrument {
        Crash = "crash" => Instrument::Crash,
        HiHat = "hiHat" => Instrument::HiHat,
        Snare = "snare" => Instrument::Snare,
        Kick = "kick" => Instrument::Kick,
    }
}

names! {
    /// How hard a hit is.
    KindName => HitKind {
        Accent = "accent" => HitKind::Accent,
        Normal = "normal" => HitKind::Normal,
        Ghost = "ghost" => HitKind::Ghost,
    }
}
