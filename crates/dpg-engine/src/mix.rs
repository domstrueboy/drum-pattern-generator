//! The mixer's settings: a level for each channel, solo and mute, and the
//! Drums and Metronome faders. The UI writes them as atomics; the audio
//! thread reads a snapshot each block and turns it into channel gains.

use std::sync::atomic::{AtomicU8, AtomicU32, Ordering};

use drum_patterns::Instrument;

#[cfg(test)]
#[path = "mix_tests.rs"]
mod tests;

/// How many channels the mixer has.
pub const CHANNELS: usize = 5;

/// Where a voice is heard: one channel per drum, and the metronome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Channel {
    /// The crash: on the Drums fader, no fader of its own on screen.
    Crash,
    /// The hi-hat.
    HiHat,
    /// The snare.
    Snare,
    /// The kick.
    Kick,
    /// The metronome's clicks, on the Metronome fader.
    Metronome,
}

impl Channel {
    /// Every channel, in index order.
    pub const ALL: [Self; CHANNELS] = [
        Self::Crash,
        Self::HiHat,
        Self::Snare,
        Self::Kick,
        Self::Metronome,
    ];

    /// Its place in [`Channel::ALL`] and in a gains array.
    pub fn index(self) -> usize {
        self as usize
    }

    fn bit(self) -> u8 {
        1 << self.index()
    }
}

impl From<Instrument> for Channel {
    fn from(instrument: Instrument) -> Self {
        match instrument {
            Instrument::Crash => Self::Crash,
            Instrument::HiHat => Self::HiHat,
            Instrument::Snare => Self::Snare,
            Instrument::Kick => Self::Kick,
        }
    }
}

/// A fader's position, 0 to 1, as a gain. Squared, so the middle of the
/// travel is -12 dB rather than -6 dB - closer to how loudness is heard.
pub fn taper(position: f32) -> f32 {
    let p = position.clamp(0.0, 1.0);
    p * p
}

/// The mixer's settings at one moment. Fader positions, 0 to 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MixSettings {
    /// Each channel's own fader. The metronome's is the Metronome fader.
    pub levels: [f32; CHANNELS],
    /// The Drums fader, over every drum channel.
    pub drums: f32,
    /// Soloed channels, one bit each by index.
    pub solo: u8,
    /// Muted channels, one bit each by index.
    pub mute: u8,
}

impl MixSettings {
    /// The starting mix: every drum fader up, Drums and Metronome a little
    /// down to leave room for several hits at once.
    pub const DEFAULT: Self = Self {
        levels: [1.0, 1.0, 1.0, 1.0, 0.8],
        drums: 0.8,
        solo: 0,
        mute: 0,
    };
}

impl Default for MixSettings {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Each channel's gain for `settings`. A muted channel is silent; while
/// anything is soloed, only soloed channels play - the crash too - and the
/// metronome keeps going either way.
pub fn gains(settings: &MixSettings) -> [f32; CHANNELS] {
    let any_solo = settings.solo & !Channel::Metronome.bit() != 0;
    Channel::ALL.map(|channel| {
        let level = taper(settings.levels[channel.index()]);
        if channel == Channel::Metronome {
            return level;
        }
        let muted = settings.mute & channel.bit() != 0;
        let soloed_out = any_solo && settings.solo & channel.bit() == 0;
        if muted || soloed_out {
            0.0
        } else {
            level * taper(settings.drums)
        }
    })
}

/// [`MixSettings`] shared between the UI, which writes, and the audio
/// thread, which reads. Atomics only.
#[derive(Debug)]
pub struct MixControls {
    levels: [AtomicU32; CHANNELS],
    drums: AtomicU32,
    solo: AtomicU8,
    mute: AtomicU8,
}

impl Default for MixControls {
    fn default() -> Self {
        let d = MixSettings::DEFAULT;
        Self {
            levels: d.levels.map(|l| AtomicU32::new(l.to_bits())),
            drums: AtomicU32::new(d.drums.to_bits()),
            solo: AtomicU8::new(d.solo),
            mute: AtomicU8::new(d.mute),
        }
    }
}

impl MixControls {
    /// Moves a channel's fader, 0 to 1.
    pub fn set_level(&self, channel: Channel, position: f32) {
        self.levels[channel.index()].store(position.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    /// Moves the Drums fader, 0 to 1.
    pub fn set_drums(&self, position: f32) {
        self.drums
            .store(position.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    /// Solos a channel, or un-solos it.
    pub fn set_solo(&self, channel: Channel, on: bool) {
        Self::set_bit(&self.solo, channel, on);
    }

    /// Mutes a channel, or un-mutes it.
    pub fn set_mute(&self, channel: Channel, on: bool) {
        Self::set_bit(&self.mute, channel, on);
    }

    /// The settings as they are now.
    pub fn snapshot(&self) -> MixSettings {
        MixSettings {
            levels: std::array::from_fn(|i| f32::from_bits(self.levels[i].load(Ordering::Relaxed))),
            drums: f32::from_bits(self.drums.load(Ordering::Relaxed)),
            solo: self.solo.load(Ordering::Relaxed),
            mute: self.mute.load(Ordering::Relaxed),
        }
    }

    fn set_bit(bits: &AtomicU8, channel: Channel, on: bool) {
        if on {
            bits.fetch_or(channel.bit(), Ordering::Relaxed);
        } else {
            bits.fetch_and(!channel.bit(), Ordering::Relaxed);
        }
    }
}
