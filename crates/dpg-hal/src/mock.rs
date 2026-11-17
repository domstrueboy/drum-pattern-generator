//! An output with no device: a test pulls frames from it as a device
//! would, so the code above runs without sound or timing.

use std::sync::Arc;

use crate::{MAX_BLOCK_FRAMES, OutputInfo, OutputStats, Render, fill};

#[cfg(test)]
#[path = "mock_tests.rs"]
mod tests;

/// An output a test drives by hand.
pub struct MockOutput<R: Render> {
    render: R,
    info: OutputInfo,
    stats: Arc<OutputStats>,
    scratch: Vec<f32>,
    latency: f64,
}

impl<R: Render> MockOutput<R> {
    /// A pretend device at `rate` with `channels`, playing `render`.
    pub fn new(rate: u32, channels: u16, render: R) -> Self {
        Self {
            render,
            info: OutputInfo {
                device: "Mock".into(),
                rate,
                channels,
            },
            stats: Arc::default(),
            scratch: vec![0.0; MAX_BLOCK_FRAMES * usize::from(channels)],
            latency: 0.0,
        }
    }

    /// The latency every block reports from now on, in seconds.
    pub fn set_latency(&mut self, seconds: f64) {
        self.latency = seconds;
        self.stats.record_latency(seconds);
    }

    /// What the device would play next: `frames` frames, interleaved,
    /// clipped.
    pub fn pull(&mut self, frames: usize) -> Vec<f32> {
        let channels = usize::from(self.info.channels);
        let mut out = vec![0.0; frames * channels];
        fill(
            &mut self.render,
            &mut self.scratch,
            channels,
            self.info.rate,
            self.latency,
            out.len(),
            |at, block| out[at..at + block.len()].copy_from_slice(block),
        );
        out
    }

    /// The pretend device.
    pub fn info(&self) -> &OutputInfo {
        &self.info
    }

    /// Its health, as a real output reports it.
    pub fn stats(&self) -> Arc<OutputStats> {
        self.stats.clone()
    }

    /// The renderer, to inspect after pulling.
    pub fn render(&self) -> &R {
        &self.render
    }
}
