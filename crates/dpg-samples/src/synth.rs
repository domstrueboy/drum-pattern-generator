//! A kit made from formulas instead of files: the app's sound until real
//! presets arrive (step 6), and one that may ship anywhere - no licensed
//! audio in it.

use std::f32::consts::TAU;

use drum_patterns::Instrument;

use crate::kit::{Kit, Sound};

#[cfg(test)]
#[path = "synth_tests.rs"]
mod tests;

/// A plain synthesized kit at `rate`: a falling sine for the kick, tone and
/// noise for the snare, short and long noise for the hi-hat and crash.
/// Each sound peaks at full scale; the same rate always gives the same kit.
pub fn placeholder_kit(rate: u32) -> Kit {
    let mut kit = Kit::new(rate);
    for instrument in Instrument::ALL {
        kit.set(instrument, Sound::new(render(instrument, rate), None));
    }
    kit
}

fn render(instrument: Instrument, rate: u32) -> Vec<f32> {
    let mut noise = Noise(0x9E37_79B9);
    let (seconds, decay) = match instrument {
        Instrument::Crash => (1.6, 0.45),
        Instrument::HiHat => (0.12, 0.025),
        Instrument::Snare => (0.3, 0.06),
        Instrument::Kick => (0.4, 0.12),
    };
    let len = (seconds * rate as f32) as usize;
    let mut phase = 0.0f32;
    let mut last = 0.0f32;
    let mut samples: Vec<f32> = (0..len)
        .map(|i| {
            let t = i as f32 / rate as f32;
            let envelope = (-t / decay).exp();
            let value = match instrument {
                Instrument::Kick => {
                    // A pitch falling from 150 Hz to 50 Hz gives the thump.
                    phase += TAU * (50.0 + 100.0 * (-t / 0.03).exp()) / rate as f32;
                    phase.sin()
                }
                Instrument::Snare => {
                    0.4 * (TAU * 190.0 * t).sin() * (-t / 0.03).exp() + 0.6 * noise.next()
                }
                // A first difference tilts the noise bright, like cymbals.
                Instrument::HiHat | Instrument::Crash => {
                    let n = noise.next();
                    let bright = n - last;
                    last = n;
                    bright
                }
            };
            value * envelope
        })
        .collect();
    let peak = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    samples.iter_mut().for_each(|s| *s /= peak);
    samples
}

/// White noise from a fixed seed (xorshift), so the kit never changes.
struct Noise(u32);

impl Noise {
    fn next(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x as f32 / u32::MAX as f32 * 2.0 - 1.0
    }
}
