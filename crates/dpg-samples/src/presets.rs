//! The Sound menu's presets: a sound for each instrument in each style,
//! made from formulas so any of them may ship. Electro and Hip-hop follow
//! the 808 and 909; Rock, Jazz and Metal are stand-ins until the author's
//! recordings replace them.

use drum_patterns::Instrument;

use crate::dsp::{BandPass, Metal, Noise, OnePole, Sweep, drive, envelope, finish, frames};
use crate::kit::Sound;

#[cfg(test)]
#[path = "presets_tests.rs"]
mod tests;

/// A style of kit, as the Sound menu lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Preset {
    /// An open acoustic kit.
    #[default]
    Rock,
    /// A 909-style drum machine.
    HipHop,
    /// An 808-style drum machine.
    Electro,
    /// Soft hits, little attack: brushes or rods.
    Jazz,
    /// Aggressive triggered drums, hard attack.
    Metal,
}

impl Preset {
    /// Every preset, in the menu's order.
    pub const ALL: [Self; 5] = [
        Self::Rock,
        Self::HipHop,
        Self::Electro,
        Self::Jazz,
        Self::Metal,
    ];
}

/// `preset`'s sound for `instrument`, made at `rate`. The same arguments
/// always give the same sound.
pub fn preset_sound(preset: Preset, instrument: Instrument, rate: u32) -> Sound {
    let rate = rate as f32;
    let samples = match instrument {
        Instrument::Kick => kick(rate, &KICKS[preset as usize]),
        Instrument::Snare => snare(rate, &SNARES[preset as usize]),
        Instrument::HiHat => cymbal(rate, &HI_HATS[preset as usize]),
        Instrument::Crash => cymbal(rate, &CRASHES[preset as usize]),
    };
    Sound::new(samples, None)
}

/// A falling sine and a beater's click. Times in ms.
struct Kick {
    from: f32,
    to: f32,
    sweep: f32,
    attack: f32,
    decay: f32,
    len: f32,
    /// The beater's click against the thump, 0 to 1.
    click: f32,
    drive: f32,
}

/// Two drum tones and the wires' noise. Times in ms.
struct Snare {
    tones: [(f32, f32); 2],
    tone_decay: f32,
    noise: f32,
    noise_hz: f32,
    noise_q: f32,
    noise_decay: f32,
    attack: f32,
    len: f32,
    drive: f32,
}

/// The 808's metal and noise, filtered bright. Times in ms.
struct Cymbal {
    metal: f32,
    pitch: f32,
    noise: f32,
    high_hz: f32,
    low_hz: f32,
    attack: f32,
    decay: f32,
    len: f32,
}

// In the menu's order: Rock, Hip-hop, Electro, Jazz, Metal.

#[rustfmt::skip]
const KICKS: [Kick; 5] = [
    Kick { from: 110.0, to: 58.0, sweep: 18.0, attack: 0.5, decay: 220.0, len: 600.0, click: 0.35, drive: 1.2 },
    Kick { from: 230.0, to: 55.0, sweep: 22.0, attack: 0.3, decay: 300.0, len: 700.0, click: 0.25, drive: 2.0 },
    Kick { from: 120.0, to: 48.0, sweep: 45.0, attack: 0.3, decay: 650.0, len: 1_500.0, click: 0.05, drive: 1.0 },
    Kick { from: 90.0, to: 70.0, sweep: 30.0, attack: 6.0, decay: 180.0, len: 500.0, click: 0.0, drive: 1.0 },
    Kick { from: 170.0, to: 55.0, sweep: 12.0, attack: 0.2, decay: 140.0, len: 400.0, click: 0.8, drive: 2.5 },
];

#[rustfmt::skip]
const SNARES: [Snare; 5] = [
    Snare { tones: [(200.0, 1.0), (290.0, 0.6)], tone_decay: 70.0, noise: 1.0, noise_hz: 4_000.0, noise_q: 0.6, noise_decay: 180.0, attack: 0.5, len: 450.0, drive: 1.3 },
    Snare { tones: [(190.0, 1.0), (340.0, 0.5)], tone_decay: 50.0, noise: 1.2, noise_hz: 6_000.0, noise_q: 0.5, noise_decay: 160.0, attack: 0.3, len: 400.0, drive: 1.5 },
    Snare { tones: [(180.0, 1.0), (330.0, 0.7)], tone_decay: 100.0, noise: 0.6, noise_hz: 7_000.0, noise_q: 0.7, noise_decay: 120.0, attack: 0.3, len: 400.0, drive: 1.0 },
    Snare { tones: [(220.0, 0.15), (330.0, 0.0)], tone_decay: 40.0, noise: 1.0, noise_hz: 3_000.0, noise_q: 0.4, noise_decay: 220.0, attack: 12.0, len: 500.0, drive: 1.0 },
    Snare { tones: [(230.0, 1.0), (340.0, 0.8)], tone_decay: 60.0, noise: 1.3, noise_hz: 5_000.0, noise_q: 0.6, noise_decay: 170.0, attack: 0.1, len: 400.0, drive: 3.0 },
];

#[rustfmt::skip]
const HI_HATS: [Cymbal; 5] = [
    Cymbal { metal: 0.6, pitch: 1.6, noise: 0.6, high_hz: 6_000.0, low_hz: 16_000.0, attack: 0.5, decay: 60.0, len: 250.0 },
    Cymbal { metal: 0.3, pitch: 1.0, noise: 1.0, high_hz: 8_000.0, low_hz: 18_000.0, attack: 0.2, decay: 40.0, len: 200.0 },
    Cymbal { metal: 1.0, pitch: 1.0, noise: 0.1, high_hz: 7_000.0, low_hz: 18_000.0, attack: 0.2, decay: 50.0, len: 220.0 },
    Cymbal { metal: 0.3, pitch: 1.4, noise: 0.8, high_hz: 5_000.0, low_hz: 9_000.0, attack: 4.0, decay: 70.0, len: 300.0 },
    Cymbal { metal: 0.5, pitch: 1.8, noise: 1.0, high_hz: 9_000.0, low_hz: 20_000.0, attack: 0.1, decay: 45.0, len: 200.0 },
];

#[rustfmt::skip]
const CRASHES: [Cymbal; 5] = [
    Cymbal { metal: 0.7, pitch: 1.3, noise: 0.8, high_hz: 3_500.0, low_hz: 15_000.0, attack: 3.0, decay: 900.0, len: 3_000.0 },
    Cymbal { metal: 0.4, pitch: 1.0, noise: 1.0, high_hz: 4_500.0, low_hz: 16_000.0, attack: 1.0, decay: 700.0, len: 2_500.0 },
    Cymbal { metal: 1.0, pitch: 0.9, noise: 0.2, high_hz: 5_000.0, low_hz: 16_000.0, attack: 1.0, decay: 800.0, len: 2_500.0 },
    Cymbal { metal: 0.6, pitch: 1.5, noise: 0.5, high_hz: 3_000.0, low_hz: 8_000.0, attack: 6.0, decay: 1_200.0, len: 3_500.0 },
    Cymbal { metal: 0.6, pitch: 1.7, noise: 1.0, high_hz: 4_000.0, low_hz: 20_000.0, attack: 0.5, decay: 1_000.0, len: 3_000.0 },
];

/// How long the end fades, so nothing stops on a click.
const FADE_MS: f32 = 20.0;

fn kick(rate: f32, k: &Kick) -> Vec<f32> {
    let mut thump = Sweep::new(k.from, k.to, k.sweep);
    let mut noise = Noise::new();
    let mut beater = BandPass::new(rate, 3_000.0, 0.8);
    let samples = (0..frames(rate, k.len))
        .map(|i| {
            let t = i as f32 * 1000.0 / rate;
            let body = thump.next(rate, t) * envelope(t, k.attack, k.decay);
            let click = beater.next(noise.next()) * (-t / 4.0).exp() * 4.0 * k.click;
            drive(body + click, k.drive)
        })
        .collect();
    finish(samples, rate, FADE_MS)
}

fn snare(rate: f32, s: &Snare) -> Vec<f32> {
    let mut noise = Noise::new();
    let mut wires = BandPass::new(rate, s.noise_hz, s.noise_q);
    let mut phases = [0.0f32; 2];
    let samples = (0..frames(rate, s.len))
        .map(|i| {
            let t = i as f32 * 1000.0 / rate;
            let mut tone = 0.0;
            for (phase, &(hz, level)) in phases.iter_mut().zip(&s.tones) {
                *phase = (*phase + hz / rate).fract();
                tone += level * (std::f32::consts::TAU * *phase).sin();
            }
            let tone = tone * envelope(t, s.attack, s.tone_decay);
            let rattle =
                wires.next(noise.next()) * 3.0 * s.noise * envelope(t, s.attack, s.noise_decay);
            drive(tone + rattle, s.drive)
        })
        .collect();
    finish(samples, rate, FADE_MS)
}

fn cymbal(rate: f32, c: &Cymbal) -> Vec<f32> {
    let mut metal = Metal::new(rate, c.pitch);
    let mut noise = Noise::new();
    let (mut high, mut high2) = (OnePole::new(rate, c.high_hz), OnePole::new(rate, c.high_hz));
    let mut low = OnePole::new(rate, c.low_hz);
    let samples = (0..frames(rate, c.len))
        .map(|i| {
            let t = i as f32 * 1000.0 / rate;
            let raw = c.metal * metal.next() + c.noise * noise.next();
            low.low(high2.high(high.high(raw))) * envelope(t, c.attack, c.decay)
        })
        .collect();
    finish(samples, rate, FADE_MS)
}
