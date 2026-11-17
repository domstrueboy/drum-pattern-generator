//! The real output, through cpal: AAudio on Android, WASAPI on Windows.
//! The stream lives on a thread of its own, since cpal streams can't move
//! between threads on every platform.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};

use crate::{HalError, MAX_BLOCK_FRAMES, OutputInfo, OutputStats, Render, fill};

#[cfg(test)]
#[path = "cpal_output_tests.rs"]
mod tests;

/// The system's default output, playing until dropped.
pub struct CpalOutput {
    info: OutputInfo,
    stats: Arc<OutputStats>,
    stop: Arc<AtomicBool>,
    owner: Option<JoinHandle<()>>,
}

impl CpalOutput {
    /// Opens the default output and starts `render` on it.
    pub fn open(render: impl Render) -> Result<Self, HalError> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or(HalError::NoDevice)?;
        let supported = device
            .default_output_config()
            .map_err(|e| HalError::Unsupported(e.to_string()))?;
        #[allow(unused_mut)]
        let mut config = supported.config();
        // AAudio's default buffer is its whole capacity (~43 ms on the test
        // phone); one 256-frame burst keeps only ~5 ms of it.
        #[cfg(target_os = "android")]
        {
            config.buffer_size = cpal::BufferSize::Fixed(256);
        }
        let info = OutputInfo {
            device: device
                .description()
                .map(|d| d.to_string())
                .unwrap_or_default(),
            rate: config.sample_rate,
            channels: config.channels,
        };

        let stats = Arc::new(OutputStats::default());
        let stop = Arc::new(AtomicBool::new(false));
        let format = supported.sample_format();
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();
        let owner = {
            let (stats, stop) = (stats.clone(), stop.clone());
            std::thread::Builder::new()
                .name("audio-output".into())
                .spawn(move || {
                    let stream = match format {
                        cpal::SampleFormat::F32 => build::<f32>(&device, &config, render, stats),
                        cpal::SampleFormat::I16 => build::<i16>(&device, &config, render, stats),
                        cpal::SampleFormat::I32 => build::<i32>(&device, &config, render, stats),
                        other => Err(HalError::Unsupported(format!("sample format {other:?}"))),
                    }
                    .and_then(|s| {
                        s.play().map_err(|e| HalError::Stream(e.to_string()))?;
                        Ok(s)
                    });
                    match stream {
                        Ok(stream) => {
                            let _ = ready_tx.send(Ok(()));
                            while !stop.load(Ordering::Acquire) {
                                std::thread::park();
                            }
                            drop(stream);
                        }
                        Err(e) => {
                            let _ = ready_tx.send(Err(e));
                        }
                    }
                })
                .map_err(|e| HalError::Stream(e.to_string()))?
        };
        ready_rx
            .recv()
            .map_err(|e| HalError::Stream(e.to_string()))??;

        Ok(Self {
            info,
            stats,
            stop,
            owner: Some(owner),
        })
    }

    /// What the device turned out to be.
    pub fn info(&self) -> &OutputInfo {
        &self.info
    }

    /// Its health: underruns, latency, whether it was lost.
    pub fn stats(&self) -> Arc<OutputStats> {
        self.stats.clone()
    }
}

impl Drop for CpalOutput {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(owner) = self.owner.take() {
            owner.thread().unpark();
            let _ = owner.join();
        }
    }
}

fn build<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut render: impl Render,
    stats: Arc<OutputStats>,
) -> Result<cpal::Stream, HalError>
where
    T: SizedSample + FromSample<f32>,
{
    let channels = usize::from(config.channels);
    let rate = config.sample_rate;
    let mut scratch = vec![0.0f32; MAX_BLOCK_FRAMES * channels];
    let errors = stats.clone();
    device
        .build_output_stream::<T, _, _>(
            *config,
            move |data: &mut [T], info: &cpal::OutputCallbackInfo| {
                let ts = info.timestamp();
                let latency = ts.playback.duration_since(ts.callback).as_secs_f64();
                stats.record_latency(latency);
                fill(
                    &mut render,
                    &mut scratch,
                    channels,
                    rate,
                    latency,
                    data.len(),
                    |at, block| {
                        for (out, &s) in data[at..at + block.len()].iter_mut().zip(block) {
                            *out = T::from_sample(s);
                        }
                    },
                );
            },
            move |err| match err.kind() {
                cpal::ErrorKind::Xrun => {
                    errors.xruns.fetch_add(1, Ordering::Relaxed);
                }
                cpal::ErrorKind::DeviceNotAvailable => {
                    errors.lost.store(true, Ordering::Relaxed);
                }
                _ => {}
            },
            None,
        )
        .map_err(|e| HalError::Stream(e.to_string()))
}
