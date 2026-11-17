//! Spike audio engine: a 16-step, 4-lane loop and tap pads, played through
//! cpal. Throwaway - it answers step 1's questions (wobble, file playback,
//! screen off, playhead sync), the real engine is `dpg-engine`.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU32, Ordering};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};

pub const LANES: usize = 4;
pub const STEPS: u32 = 16;
const VOICES: usize = 24;
const SAMPLE_LANE: usize = 3;

/// State shared between the UI side and the audio callback - atomics only.
pub struct Shared {
    pub playing: AtomicBool,
    bpm_bits: AtomicU32,
    pub lanes: [AtomicU16; LANES],
    /// Pads hit since the last callback, one bit per lane.
    hits: AtomicU32,
    /// Bar position of what is heard right now, 0..1, as `f32` bits.
    phase_bits: AtomicU32,
    latency_us: AtomicU32,
    pub xruns: AtomicU32,
}

impl Shared {
    fn new() -> Self {
        // Bit n = 16th step n. A plain rock beat to start from.
        let steps = |s: &[u32]| AtomicU16::new(s.iter().fold(0, |m, &i| m | 1 << i));
        let lanes = [
            steps(&[0, 8, 10]),                // kick: 1, 3, 3-and
            steps(&[4, 12]),                   // snare: 2, 4
            steps(&[0, 2, 4, 6, 8, 10, 12, 14]), // hi-hat: 8ths
            steps(&[]),                        // the loaded sample
        ];
        Self {
            playing: AtomicBool::new(false),
            bpm_bits: AtomicU32::new(120f32.to_bits()),
            lanes,
            hits: AtomicU32::new(0),
            phase_bits: AtomicU32::new(0),
            latency_us: AtomicU32::new(0),
            xruns: AtomicU32::new(0),
        }
    }

    pub fn set_bpm(&self, bpm: f32) {
        self.bpm_bits.store(bpm.clamp(30.0, 300.0).to_bits(), Ordering::Relaxed);
    }

    pub fn bpm(&self) -> f32 {
        f32::from_bits(self.bpm_bits.load(Ordering::Relaxed))
    }

    pub fn hit(&self, lane: usize) {
        self.hits.fetch_or(1 << lane, Ordering::Relaxed);
    }

    pub fn phase(&self) -> f32 {
        f32::from_bits(self.phase_bits.load(Ordering::Relaxed))
    }

    pub fn latency_ms(&self) -> f32 {
        self.latency_us.load(Ordering::Relaxed) as f32 / 1000.0
    }
}

pub struct EngineInfo {
    pub device: String,
    pub sample_rate: u32,
    pub channels: u16,
}

/// The UI side's handle: sends loaded samples in, frees replaced ones.
pub struct Engine {
    pub shared: Arc<Shared>,
    pub info: EngineInfo,
    to_audio: rtrb::Producer<Box<[f32]>>,
    from_audio: rtrb::Consumer<Box<[f32]>>,
}

impl Engine {
    /// Opens the default output and starts the stream on a thread that owns it.
    pub fn start() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or("no output device")?;
        let supported = device.default_output_config().map_err(|e| e.to_string())?;
        #[allow(unused_mut)]
        let mut config = supported.config();
        // AAudio's default client buffer is its whole capacity (2048 frames,
        // ~43 ms on the test phone). One mixer burst keeps only ~5 ms.
        #[cfg(target_os = "android")]
        {
            config.buffer_size = cpal::BufferSize::Fixed(256);
        }
        let info = EngineInfo {
            device: device.description().map(|d| d.to_string()).unwrap_or_default(),
            sample_rate: config.sample_rate,
            channels: config.channels,
        };

        let shared = Arc::new(Shared::new());
        let (to_audio, inbox) = rtrb::RingBuffer::new(4);
        let (outbox, from_audio) = rtrb::RingBuffer::new(4);
        let core = Core::new(shared.clone(), config.sample_rate, config.channels, inbox, outbox);
        let format = supported.sample_format();

        let (ready_tx, ready_rx) = std::sync::mpsc::channel();
        std::thread::Builder::new()
            .name("audio-owner".into())
            .spawn(move || {
                let built = match format {
                    cpal::SampleFormat::F32 => build::<f32>(&device, &config, core),
                    cpal::SampleFormat::I16 => build::<i16>(&device, &config, core),
                    cpal::SampleFormat::I32 => build::<i32>(&device, &config, core),
                    other => Err(format!("unsupported sample format {other:?}")),
                };
                match built.and_then(|s| s.play().map(|_| s).map_err(|e| e.to_string())) {
                    Ok(stream) => {
                        let _ = ready_tx.send(Ok(()));
                        // The stream lives as long as this thread, i.e. the app.
                        loop {
                            std::thread::park();
                            let _ = &stream;
                        }
                    }
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        ready_rx.recv().map_err(|e| e.to_string())??;

        Ok(Self { shared, info, to_audio, from_audio })
    }

    /// Hands a mono buffer at the device rate to the audio thread.
    pub fn set_sample(&mut self, samples: Box<[f32]>) -> Result<(), String> {
        while self.from_audio.pop().is_ok() {} // drop replaced buffers here, not there
        self.to_audio.push(samples).map_err(|_| "audio thread busy, try again".to_string())
    }
}

fn build<T>(device: &cpal::Device, config: &cpal::StreamConfig, mut core: Core) -> Result<cpal::Stream, String>
where
    T: SizedSample + FromSample<f32>,
{
    let channels = config.channels as usize;
    let shared = core.shared.clone();
    device
        .build_output_stream::<T, _, _>(
            config.clone(),
            move |data: &mut [T], info: &cpal::OutputCallbackInfo| {
                let ts = info.timestamp();
                let latency = ts.playback.duration_since(ts.callback);
                core.render(data, channels, latency.as_secs_f64());
            },
            move |err| {
                if matches!(err.kind(), cpal::ErrorKind::Xrun) {
                    shared.xruns.fetch_add(1, Ordering::Relaxed);
                }
            },
            None,
        )
        .map_err(|e| e.to_string())
}

#[derive(Clone, Copy)]
struct Voice {
    sound: usize,
    pos: usize,
    active: bool,
}

/// Everything the audio callback owns. Preallocated; `render` never allocates.
struct Core {
    shared: Arc<Shared>,
    rate: f64,
    sounds: [Box<[f32]>; LANES],
    voices: [Voice; VOICES],
    inbox: rtrb::Consumer<Box<[f32]>>,
    outbox: rtrb::Producer<Box<[f32]>>,
    was_playing: bool,
    frame: u64,
    step: u32,
    step_start: f64,
    next_step: f64,
    frames_per_step: f64,
}

impl Core {
    fn new(
        shared: Arc<Shared>,
        rate: u32,
        _channels: u16,
        inbox: rtrb::Consumer<Box<[f32]>>,
        outbox: rtrb::Producer<Box<[f32]>>,
    ) -> Self {
        let r = rate as f32;
        Self {
            shared,
            rate: rate as f64,
            sounds: [synth_kick(r), synth_snare(r), synth_hat(r), synth_blip(r)],
            voices: [Voice { sound: 0, pos: 0, active: false }; VOICES],
            inbox,
            outbox,
            was_playing: false,
            frame: 0,
            step: 0,
            step_start: 0.0,
            next_step: 0.0,
            frames_per_step: 1.0,
        }
    }

    fn trigger(&mut self, sound: usize) {
        // Steal the oldest voice when all are busy.
        let slot = self
            .voices
            .iter()
            .position(|v| !v.active)
            .unwrap_or_else(|| {
                let mut oldest = 0;
                for (i, v) in self.voices.iter().enumerate() {
                    if v.pos > self.voices[oldest].pos {
                        oldest = i;
                    }
                }
                oldest
            });
        self.voices[slot] = Voice { sound, pos: 0, active: true };
    }

    fn frames_per_step_from_bpm(&self) -> f64 {
        self.rate * 60.0 / (self.shared.bpm() as f64 * 4.0)
    }

    fn render<T: SizedSample + FromSample<f32>>(&mut self, data: &mut [T], channels: usize, latency_s: f64) {
        if let Ok(new) = self.inbox.pop() {
            let old = std::mem::replace(&mut self.sounds[SAMPLE_LANE], new);
            let _ = self.outbox.push(old); // freed on the UI side
        }

        let hits = self.shared.hits.swap(0, Ordering::Relaxed);
        for lane in 0..LANES {
            if hits & (1 << lane) != 0 {
                self.trigger(lane);
            }
        }

        let playing = self.shared.playing.load(Ordering::Relaxed);
        if playing && !self.was_playing {
            self.frame = 0;
            self.step = STEPS - 1;
            self.next_step = 0.0;
            self.step_start = 0.0;
        }
        self.was_playing = playing;

        // Publish where the heard sound is: the bar position at this buffer's
        // start, moved back by the output latency.
        if playing {
            let fps = self.frames_per_step.max(1.0);
            let in_step = ((self.frame as f64 - self.step_start) / fps).clamp(0.0, 1.0);
            let at = (self.step as f64 + in_step) / STEPS as f64;
            let back = latency_s * self.rate / (fps * STEPS as f64);
            let phase = (at - back).rem_euclid(1.0) as f32;
            self.shared.phase_bits.store(phase.to_bits(), Ordering::Relaxed);
        }
        self.shared.latency_us.store((latency_s * 1e6) as u32, Ordering::Relaxed);

        for frame in data.chunks_mut(channels) {
            if playing && self.frame as f64 >= self.next_step {
                self.step = (self.step + 1) % STEPS;
                self.frames_per_step = self.frames_per_step_from_bpm();
                self.step_start = self.next_step;
                self.next_step += self.frames_per_step;
                for lane in 0..LANES {
                    if self.shared.lanes[lane].load(Ordering::Relaxed) & (1 << self.step) != 0 {
                        self.trigger(lane);
                    }
                }
            }

            let mut mix = 0.0f32;
            for v in self.voices.iter_mut().filter(|v| v.active) {
                let sound = &self.sounds[v.sound];
                if v.pos < sound.len() {
                    mix += sound[v.pos];
                    v.pos += 1;
                } else {
                    v.active = false;
                }
            }
            let out = T::from_sample(mix.clamp(-1.0, 1.0) * 0.8);
            frame.fill(out);
            if playing {
                self.frame += 1;
            }
        }
    }
}

// Built-in sounds, synthesized so the spike needs no audio files.

struct Noise(u32);

impl Noise {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0 as f32 / u32::MAX as f32 * 2.0 - 1.0
    }
}

fn render_sound(rate: f32, seconds: f32, f: impl Fn(f32, usize) -> f32) -> Box<[f32]> {
    (0..(rate * seconds) as usize).map(|i| f(i as f32 / rate, i)).collect()
}

fn synth_kick(rate: f32) -> Box<[f32]> {
    let mut phase = 0.0f32;
    let mut out = Vec::new();
    for i in 0..(rate * 0.45) as usize {
        let t = i as f32 / rate;
        let freq = 45.0 + 110.0 * (-t * 30.0).exp();
        phase += std::f32::consts::TAU * freq / rate;
        out.push(phase.sin() * (-t * 7.0).exp());
    }
    out.into()
}

fn synth_snare(rate: f32) -> Box<[f32]> {
    let mut noise = Noise(0x1234_5678);
    let noise: Vec<f32> = (0..(rate * 0.25) as usize).map(|_| noise.next()).collect();
    render_sound(rate, 0.25, |t, i| {
        let body = (std::f32::consts::TAU * 185.0 * t).sin() * (-t * 25.0).exp();
        0.6 * noise[i] * (-t * 18.0).exp() + 0.5 * body
    })
}

fn synth_hat(rate: f32) -> Box<[f32]> {
    let mut n = Noise(0x9e37_79b9);
    let mut prev = 0.0;
    let mut out = Vec::new();
    for i in 0..(rate * 0.06) as usize {
        let t = i as f32 / rate;
        let x = n.next();
        out.push((x - prev) * 0.35 * (-t * 70.0).exp()); // first difference: crude high-pass
        prev = x;
    }
    out.into()
}

fn synth_blip(rate: f32) -> Box<[f32]> {
    render_sound(rate, 0.08, |t, _| (std::f32::consts::TAU * 1500.0 * t).sin() * 0.5 * (-t * 50.0).exp())
}

/// Decodes a WAV to mono `f32` at `rate`. Runs on a command thread, never the audio thread.
pub fn decode_wav(bytes: &[u8], rate: u32) -> Result<(Box<[f32]>, String), String> {
    let mut reader = hound::WavReader::new(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())?;
    let spec = reader.spec();
    let ch = spec.channels as usize;
    let interleaved: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().collect::<Result<_, _>>(),
        hound::SampleFormat::Int => {
            let scale = 1.0 / (1u64 << (spec.bits_per_sample - 1)) as f32;
            reader.samples::<i32>().map(|s| s.map(|v| v as f32 * scale)).collect()
        }
    }
    .map_err(|e| e.to_string())?;
    let mono: Vec<f32> = interleaved.chunks(ch).map(|f| f.iter().sum::<f32>() / ch as f32).collect();

    // Linear resampling: good enough to judge the spike, not the product.
    let ratio = spec.sample_rate as f64 / rate as f64;
    let len = ((mono.len() as f64 / ratio) as usize).min(rate as usize * 10);
    let out: Box<[f32]> = (0..len)
        .map(|i| {
            let x = i as f64 * ratio;
            let j = x as usize;
            let frac = (x - j as f64) as f32;
            let a = mono.get(j).copied().unwrap_or(0.0);
            let b = mono.get(j + 1).copied().unwrap_or(a);
            a + (b - a) * frac
        })
        .collect();
    let desc = format!(
        "{} ch, {} Hz, {}-bit, {:.2} s",
        spec.channels,
        spec.sample_rate,
        spec.bits_per_sample,
        out.len() as f32 / rate as f32
    );
    Ok((out, desc))
}
