//! A kit: a sound for each instrument, all at the device's rate. A sound
//! holds one or two samples and knows how loud each kind of hit plays.

use std::fmt;

use drum_patterns::{HitKind, Instrument};

use crate::decode::{Audio, DecodeError, decode};
use crate::level::level;
use crate::resample::{ResampleError, resample};
use crate::trim::trim;

#[cfg(test)]
#[path = "kit_tests.rs"]
mod tests;

/// An accent's level, in dB from full scale.
pub const ACCENT_DB: f32 = 0.0;
/// A normal hit's level: below the accent, so the accent has room above it.
pub const NORMAL_DB: f32 = -4.0;
/// A ghost's level when the instrument has no ghost sample of its own.
pub const GHOST_DB: f32 = -14.0;

/// An audio file as the app reads it: its bytes, and its extension if
/// known.
#[derive(Debug, Clone, Copy)]
pub struct SampleFile<'a> {
    /// The whole file.
    pub bytes: &'a [u8],
    /// "wav", "m4a"... - a hint only.
    pub extension: Option<&'a str>,
}

/// Why a sound couldn't be loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    /// The file couldn't be read as audio.
    Decode(DecodeError),
    /// It couldn't be brought to the device's rate.
    Resample(ResampleError),
    /// It holds nothing but silence.
    Silent,
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decode(e) => e.fmt(f),
            Self::Resample(e) => e.fmt(f),
            Self::Silent => write!(f, "the file is silent"),
        }
    }
}

impl std::error::Error for LoadError {}

/// One instrument's samples at the kit's rate, full scale at +/-1.0.
#[derive(Debug, Clone, PartialEq)]
pub struct Sound {
    normal: Vec<f32>,
    ghost: Option<Vec<f32>>,
}

impl Sound {
    /// A sound from samples already at the kit's rate.
    pub fn new(normal: Vec<f32>, ghost: Option<Vec<f32>>) -> Self {
        Self { normal, ghost }
    }

    /// Loads a sound from its normal sample and, if given, its ghost
    /// sample: decoded, brought to `rate`, trimmed, then levelled as one so
    /// the ghost keeps its recorded quietness.
    pub fn load(
        normal: SampleFile<'_>,
        ghost: Option<SampleFile<'_>>,
        rate: u32,
    ) -> Result<Self, LoadError> {
        let mut normal = prepare(normal, rate)?;
        let mut ghost = ghost.map(|g| prepare(g, rate)).transpose()?;
        level(std::iter::once(&mut normal).chain(ghost.as_mut()));
        Ok(Self::new(normal.samples, ghost.map(|g| g.samples)))
    }

    /// What a hit of `kind` plays: a sample and the gain to play it at.
    /// With one sample its level tells the kinds apart; with two the ghost
    /// gets its own, at the normal hit's gain.
    pub fn hit(&self, kind: HitKind) -> (&[f32], f32) {
        match (kind, &self.ghost) {
            (HitKind::Accent, _) => (&self.normal, gain(ACCENT_DB)),
            (HitKind::Normal, _) => (&self.normal, gain(NORMAL_DB)),
            (HitKind::Ghost, Some(ghost)) => (ghost, gain(NORMAL_DB)),
            (HitKind::Ghost, None) => (&self.normal, gain(GHOST_DB)),
        }
    }
}

/// A sound for each instrument, all at one rate.
#[derive(Debug, Clone, PartialEq)]
pub struct Kit {
    rate: u32,
    sounds: [Option<Sound>; 4],
}

impl Kit {
    /// A kit with no sounds yet, for a device running at `rate`.
    pub fn new(rate: u32) -> Self {
        Self {
            rate,
            sounds: Default::default(),
        }
    }

    /// The rate every sound in the kit is at.
    pub fn rate(&self) -> u32 {
        self.rate
    }

    /// The instrument's sound, if it has one.
    pub fn sound(&self, instrument: Instrument) -> Option<&Sound> {
        self.sounds[index(instrument)].as_ref()
    }

    /// Gives the instrument `sound`, replacing any it had. The sound must
    /// be at the kit's rate - as [`Sound::load`] with [`Kit::rate`] makes it.
    pub fn set(&mut self, instrument: Instrument, sound: Sound) {
        self.sounds[index(instrument)] = Some(sound);
    }
}

fn index(instrument: Instrument) -> usize {
    Instrument::ALL
        .iter()
        .position(|&i| i == instrument)
        .expect("ALL lists every instrument")
}

fn gain(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}

fn prepare(file: SampleFile<'_>, rate: u32) -> Result<Audio, LoadError> {
    let audio = decode(file.bytes, file.extension).map_err(LoadError::Decode)?;
    let audio = resample(audio, rate).map_err(LoadError::Resample)?;
    trim(audio).ok_or(LoadError::Silent)
}
