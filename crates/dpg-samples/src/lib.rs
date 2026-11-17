//! Audio files to ready-to-play drum sounds: decoded, resampled to the
//! device's rate, trimmed and levelled, off the audio thread. Takes bytes,
//! not paths - Android hands the app a content link, not a file.

mod decode;
mod level;
mod resample;
mod trim;

#[cfg(test)]
mod test_audio;

pub use decode::{Audio, DecodeError, MAX_SECONDS, decode};
pub use level::level;
pub use resample::{ResampleError, resample};
pub use trim::trim;
