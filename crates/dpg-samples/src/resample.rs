//! Converting a sound to the device's sample rate, so the audio thread
//! plays it frame for frame.

use std::fmt;

use rubato::audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Resampler};

use crate::decode::Audio;

#[cfg(test)]
#[path = "resample_tests.rs"]
mod tests;

/// Why a sound couldn't be resampled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResampleError(String);

impl fmt::Display for ResampleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "can't resample: {}", self.0)
    }
}

impl std::error::Error for ResampleError {}

/// `audio` at `rate` frames a second. The same rate passes through as it
/// is. The filter's delay is taken off, so a hit stays where it was in
/// time.
pub fn resample(audio: Audio, rate: u32) -> Result<Audio, ResampleError> {
    if audio.rate == rate {
        return Ok(audio);
    }
    let error = |e: &dyn std::error::Error| ResampleError(e.to_string());
    let frames = audio.samples.len();
    let mut resampler =
        Fft::<f32>::new(audio.rate as usize, rate as usize, 1024, 1, FixedSync::Both)
            .map_err(|e| error(&e))?;
    let input = InterleavedSlice::new(&audio.samples, 1, frames).map_err(|e| error(&e))?;
    let output = resampler
        .process_all(&input, frames, None)
        .map_err(|e| error(&e))?;
    Ok(Audio {
        rate,
        samples: output.take_data(),
    })
}
