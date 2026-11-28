//! The metronome's sounds, synthesized at the output's rate: each has a
//! higher one for the bar's first beat and a lower one for the rest. All
//! are made when the engine is built, never on the audio thread.

use std::f32::consts::TAU;

use crate::metronome::{Click, SUBDIVISION_DB};

#[cfg(test)]
#[path = "clicks_tests.rs"]
mod tests;

/// The metronome's sound: the Sound button's choices, in its order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ClickSound {
    /// A short pitched click.
    #[default]
    Click,
    /// A dry wooden tick, as Ableton's metronome.
    Ableton,
    /// A drum machine's cowbell, as the 808's.
    Cowbell,
    /// A high clave, as the CR-78's.
    Clave,
    /// A bongo: a small drum that drops in pitch.
    Bongo,
    /// A bright electronic beep that cuts through an acoustic kit, as a
    /// drum machine's metronome (the user's Nux DP-2000).
    Electronic,
}

impl ClickSound {
    /// Every sound, in the button's order.
    pub const ALL: [Self; 6] = [
        Self::Click,
        Self::Ableton,
        Self::Cowbell,
        Self::Clave,
        Self::Bongo,
        Self::Electronic,
    ];

    /// The one after it on the button; after the last, the first.
    pub fn next(self) -> Self {
        Self::ALL[(usize::from(self.index()) + 1) % Self::ALL.len()]
    }

    /// Its place in [`ClickSound::ALL`], to keep it in an atomic.
    pub(crate) fn index(self) -> u8 {
        self as u8
    }

    pub(crate) fn from_index(index: u8) -> Self {
        Self::ALL[usize::from(index).min(Self::ALL.len() - 1)]
    }
}

/// Every sound's two samples: the bar's first beat, then the others.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Clicks {
    sounds: Vec<[Vec<f32>; 2]>,
}

impl Clicks {
    pub(crate) fn new(rate: u32) -> Self {
        let rate = rate as f32;
        Self {
            sounds: ClickSound::ALL
                .iter()
                .map(|&sound| [make(sound, rate, true), make(sound, rate, false)])
                .collect(),
        }
    }

    /// What `click` plays in `sound`: a sample and its gain.
    pub(crate) fn sound(&self, sound: ClickSound, click: Click) -> (&[f32], f32) {
        let [high, low] = &self.sounds[usize::from(sound.index())];
        match click {
            Click::Bar => (high, 1.0),
            Click::Beat => (low, 1.0),
            Click::Between => (low, 10f32.powf(SUBDIVISION_DB / 20.0)),
        }
    }
}

/// One sample of `sound`, the bar's (`high`) or the others', its peak at
/// full scale.
fn make(sound: ClickSound, rate: f32, high: bool) -> Vec<f32> {
    let pick = |bar: f32, beat: f32| if high { bar } else { beat };
    let samples = match sound {
        ClickSound::Click => partials(rate, 40.0, &[(pick(2_000.0, 1_000.0), 1.0, 6.0)]),
        ClickSound::Ableton => {
            let f = pick(1_600.0, 1_100.0);
            partials(
                rate,
                30.0,
                &[(f, 1.0, 4.0), (f * 2.76, 0.5, 2.5), (f * 5.4, 0.25, 1.5)],
            )
        }
        ClickSound::Cowbell => cowbell(rate, pick(1.19, 1.0)),
        ClickSound::Clave => partials(rate, 80.0, &[(pick(3_150.0, 2_500.0), 1.0, 12.0)]),
        ClickSound::Bongo => bongo(rate, pick(440.0, 330.0)),
        ClickSound::Electronic => beep(rate, pick(1_760.0, 1_320.0)),
    };
    normalized(samples)
}

fn frames(rate: f32, ms: f32) -> usize {
    (rate * ms / 1000.0) as usize
}

/// Sines that die away, each `(hz, level, decay ms)`, `ms` long.
fn partials(rate: f32, ms: f32, parts: &[(f32, f32, f32)]) -> Vec<f32> {
    (0..frames(rate, ms))
        .map(|i| {
            let t = i as f32 / rate;
            parts
                .iter()
                .map(|&(hz, level, decay)| {
                    level * (TAU * hz * t).sin() * (-t * 1000.0 / decay).exp()
                })
                .sum()
        })
        .collect()
}

/// A square wave without the harmonics a click doesn't need: odd ones up
/// to the 7th, so nothing folds back at 44.1 kHz.
fn square(phase: f32) -> f32 {
    [1.0f32, 3.0, 5.0, 7.0]
        .iter()
        .map(|&n| (TAU * n * phase).sin() / n)
        .sum()
}

/// Two squares a fifth-ish apart (540 and 800 Hz on the 808), a sharp hit
/// then a short ring; `pitch` lifts both for the bar.
fn cowbell(rate: f32, pitch: f32) -> Vec<f32> {
    let n = frames(rate, 400.0);
    let fade = frames(rate, 10.0);
    (0..n)
        .map(|i| {
            let t = i as f32 / rate;
            let tone = square(540.0 * pitch * t) + square(800.0 * pitch * t);
            let level = 0.6 * (-t / 0.008).exp() + 0.4 * (-t / 0.09).exp();
            let tail = ((n - i) as f32 / fade as f32).min(1.0);
            tone * level * tail
        })
        .collect()
}

/// A drum's sine that starts a quarter sharp and settles on `hz`.
fn bongo(rate: f32, hz: f32) -> Vec<f32> {
    let mut phase = 0.0f32;
    (0..frames(rate, 250.0))
        .map(|i| {
            let t = i as f32 / rate;
            phase += hz * (1.0 + 0.25 * (-t / 0.01).exp()) / rate;
            (TAU * phase).sin() * (-t / 0.045).exp()
        })
        .collect()
}

/// A square beep that holds for 30 ms, then stops fast: no ring to blur
/// into the drums.
fn beep(rate: f32, hz: f32) -> Vec<f32> {
    let (attack, hold) = (0.001, 0.03);
    (0..frames(rate, 50.0))
        .map(|i| {
            let t = i as f32 / rate;
            let level = if t < attack {
                t / attack
            } else if t < hold {
                1.0
            } else {
                (-(t - hold) / 0.004).exp()
            };
            square(hz * t) * level
        })
        .collect()
}

fn normalized(mut samples: Vec<f32>) -> Vec<f32> {
    let peak = samples.iter().fold(0f32, |m, x| m.max(x.abs()));
    if peak > 0.0 {
        samples.iter_mut().for_each(|x| *x /= peak);
    }
    samples
}
