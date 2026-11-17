//! Audio files to ready-to-play drum sounds: decoded, resampled to the
//! device's rate, trimmed and levelled, off the audio thread. Takes bytes,
//! not paths - Android hands the app a content link, not a file.

mod decode;
mod resample;
#[cfg(test)]
mod test_audio;

pub use decode::{Audio, DecodeError, MAX_SECONDS, decode};
pub use resample::{ResampleError, resample};
