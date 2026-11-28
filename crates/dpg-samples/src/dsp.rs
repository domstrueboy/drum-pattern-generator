//! What the presets are made of: noise, envelopes, filters and the
//! oscillators drum machines use. Deterministic: the same calls give the
//! same samples, so a preset never changes between launches.

use std::f32::consts::TAU;

/// White noise from a fixed seed (xorshift).
pub(crate) struct Noise(u32);

impl Noise {
    pub(crate) fn new() -> Self {
        Self(0x9E37_79B9)
    }

    pub(crate) fn next(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x as f32 / u32::MAX as f32 * 2.0 - 1.0
    }
}

/// Frames in `ms` at `rate`.
pub(crate) fn frames(rate: f32, ms: f32) -> usize {
    (rate * ms / 1000.0) as usize
}

/// Rises in a straight line for `attack` ms, then dies away by e every
/// `decay` ms.
pub(crate) fn envelope(t_ms: f32, attack: f32, decay: f32) -> f32 {
    if t_ms < attack {
        t_ms / attack
    } else {
        (-(t_ms - attack) / decay).exp()
    }
}

/// A one-pole filter: gentle, 6 dB an octave.
pub(crate) struct OnePole {
    a: f32,
    y: f32,
}

impl OnePole {
    pub(crate) fn new(rate: f32, hz: f32) -> Self {
        let hz = hz.min(0.45 * rate);
        Self {
            a: 1.0 - (-TAU * hz / rate).exp(),
            y: 0.0,
        }
    }

    pub(crate) fn low(&mut self, x: f32) -> f32 {
        self.y += self.a * (x - self.y);
        self.y
    }

    pub(crate) fn high(&mut self, x: f32) -> f32 {
        x - self.low(x)
    }
}

/// A band-pass around `hz`, `q` wide (RBJ's, 0 dB at the centre).
pub(crate) struct BandPass {
    b0: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BandPass {
    pub(crate) fn new(rate: f32, hz: f32, q: f32) -> Self {
        let w = TAU * hz.min(0.45 * rate) / rate;
        let alpha = w.sin() / (2.0 * q);
        let a0 = 1.0 + alpha;
        Self {
            b0: alpha / a0,
            a1: -2.0 * w.cos() / a0,
            a2: (1.0 - alpha) / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub(crate) fn next(&mut self, x: f32) -> f32 {
        let y = self.b0 * (x - self.x2) - self.a1 * self.y1 - self.a2 * self.y2;
        (self.x2, self.x1) = (self.x1, x);
        (self.y2, self.y1) = (self.y1, y);
        y
    }
}

/// A sine whose pitch falls from `from` to `to` Hz, most of the way in
/// `sweep` ms: a drum's thump.
pub(crate) struct Sweep {
    phase: f32,
    from: f32,
    to: f32,
    sweep: f32,
}

impl Sweep {
    pub(crate) fn new(from: f32, to: f32, sweep: f32) -> Self {
        Self {
            phase: 0.0,
            from,
            to,
            sweep,
        }
    }

    pub(crate) fn next(&mut self, rate: f32, t_ms: f32) -> f32 {
        let hz = self.to + (self.from - self.to) * (-t_ms / self.sweep).exp();
        self.phase = (self.phase + hz / rate).fract();
        (TAU * self.phase).sin()
    }
}

/// The 808's cymbal tone: six square waves at pitches that clash, heard
/// as metal once filtered bright. `scale` moves them all.
pub(crate) struct Metal {
    phases: [f32; 6],
    steps: [f32; 6],
}

impl Metal {
    const HZ: [f32; 6] = [205.3, 304.4, 369.6, 522.7, 540.0, 800.0];

    pub(crate) fn new(rate: f32, scale: f32) -> Self {
        Self {
            phases: [0.0; 6],
            steps: Self::HZ.map(|hz| hz * scale / rate),
        }
    }

    pub(crate) fn next(&mut self) -> f32 {
        let mut sum = 0.0;
        for (phase, step) in self.phases.iter_mut().zip(self.steps) {
            *phase = (*phase + step).fract();
            sum += if *phase < 0.5 { 1.0 } else { -1.0 };
        }
        sum / 6.0
    }
}

/// Soft clipping: louder in the body, the peaks rounded. 1 leaves it
/// almost clean; more is harder.
pub(crate) fn drive(x: f32, amount: f32) -> f32 {
    (x * amount).tanh()
}

/// Peak at full scale, and the last `ms` faded to silence so no sound
/// stops on a click.
pub(crate) fn finish(mut samples: Vec<f32>, rate: f32, ms: f32) -> Vec<f32> {
    let peak = samples.iter().fold(0f32, |m, x| m.max(x.abs()));
    if peak > 0.0 {
        samples.iter_mut().for_each(|x| *x /= peak);
    }
    let fade = frames(rate, ms).min(samples.len());
    let len = samples.len();
    for (i, x) in samples[len - fade..].iter_mut().enumerate() {
        // Curved, so the very end is quieter than a straight line leaves it.
        *x *= (1.0 - i as f32 / fade as f32).powi(2);
    }
    samples
}
