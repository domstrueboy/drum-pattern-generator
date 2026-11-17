//! Audio output: the device, its stream and its format, and nothing about
//! what plays through it. Samples cross this boundary as `f32`, full scale
//! at +/-1.0, and are clipped here, once, on the way out.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

#[cfg(feature = "cpal")]
mod cpal_output;
mod mock;

#[cfg(feature = "cpal")]
pub use cpal_output::CpalOutput;
pub use mock::MockOutput;

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

/// The most frames a [`Render`] is handed at once. A device asking for
/// more gets several blocks, so buffers can be sized up front.
pub const MAX_BLOCK_FRAMES: usize = 4096;

/// What fills the output. Runs on the audio thread: no allocation, no
/// locks, no blocking.
pub trait Render: Send + 'static {
    /// Fills `block.out`, interleaved, with silence already in it.
    fn render(&mut self, block: Block<'_>);
}

/// One block of output to fill.
#[derive(Debug)]
pub struct Block<'a> {
    /// Interleaved frames, `channels` samples each, zeroed.
    pub out: &'a mut [f32],
    /// Samples a frame.
    pub channels: usize,
    /// Seconds from now until the block's first frame is heard.
    pub latency: f64,
}

/// What an opened output turned out to be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputInfo {
    /// The device's name, for display.
    pub device: String,
    /// Frames a second.
    pub rate: u32,
    /// Samples a frame.
    pub channels: u16,
}

/// Why an output couldn't be opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HalError {
    /// The system has no output device.
    NoDevice,
    /// The device offers nothing this crate can use.
    Unsupported(String),
    /// The device refused the stream.
    Stream(String),
}

impl std::fmt::Display for HalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoDevice => write!(f, "no audio output"),
            Self::Unsupported(why) => write!(f, "unsupported audio output: {why}"),
            Self::Stream(why) => write!(f, "can't start audio: {why}"),
        }
    }
}

impl std::error::Error for HalError {}

/// Health of a running output, readable from any thread.
#[derive(Debug, Default)]
pub struct OutputStats {
    xruns: AtomicU32,
    latency_us: AtomicU32,
    lost: AtomicBool,
}

impl OutputStats {
    /// Blocks the device wanted before they were ready - heard as clicks.
    pub fn xruns(&self) -> u32 {
        self.xruns.load(Ordering::Relaxed)
    }

    /// The latest output latency, in seconds.
    pub fn latency(&self) -> f64 {
        f64::from(self.latency_us.load(Ordering::Relaxed)) / 1e6
    }

    /// The device went away; the output must be opened again.
    pub fn lost(&self) -> bool {
        self.lost.load(Ordering::Relaxed)
    }

    fn record_latency(&self, seconds: f64) {
        let us = (seconds * 1e6).clamp(0.0, f64::from(u32::MAX)) as u32;
        self.latency_us.store(us, Ordering::Relaxed);
    }
}

/// Clips to full scale - the one place it happens.
fn clip(samples: &mut [f32]) {
    for s in samples {
        *s = s.clamp(-1.0, 1.0);
    }
}

/// Renders `samples` interleaved samples in blocks of at most
/// [`MAX_BLOCK_FRAMES`] through `scratch` (sized for one block), clips
/// each, and hands it to `write` with its offset. The loop every output
/// shares; allocates nothing.
fn fill(
    render: &mut impl Render,
    scratch: &mut [f32],
    channels: usize,
    rate: u32,
    latency: f64,
    samples: usize,
    mut write: impl FnMut(usize, &[f32]),
) {
    let mut start = 0;
    while start < samples {
        let len = (samples - start).min(scratch.len());
        let block = &mut scratch[..len];
        block.fill(0.0);
        let offset = (start / channels) as f64 / f64::from(rate);
        render.render(Block {
            out: block,
            channels,
            latency: latency + offset,
        });
        clip(block);
        write(start, block);
        start += len;
    }
}
