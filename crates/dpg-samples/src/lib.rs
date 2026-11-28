//! Audio files to ready-to-play drum sounds: decoded, resampled to the
//! device's rate, trimmed and levelled, off the audio thread. Takes bytes,
//! not paths - Android hands the app a content link, not a file. Also the
//! Sound menu's presets, synthesized.

mod decode;
mod kit;
mod level;
mod presets;
mod resample;
mod synth;
mod trim;

#[cfg(test)]
mod test_audio;

pub use decode::{Audio, DecodeError, MAX_SECONDS, decode};
pub use kit::{ACCENT_DB, GHOST_DB, Kit, LoadError, NORMAL_DB, SampleFile, Sound};
pub use level::level;
pub use presets::{Preset, preset_sound};
pub use resample::{ResampleError, resample};
pub use synth::placeholder_kit;
pub use trim::trim;
