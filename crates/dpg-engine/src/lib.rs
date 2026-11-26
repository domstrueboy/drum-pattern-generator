//! Plays grooves: what sounds when, how loud, on the audio thread. Knows
//! no device - it fills the blocks `dpg-hal` hands it.

mod clock;
mod mix;
mod voices;

pub use clock::{BPM_MAX, BPM_MIN, Clock, Tick};
pub use mix::{CHANNELS, Channel, MixControls, MixSettings, gains, taper};
pub use voices::{FADE_MS, SampleId, Trigger, VOICES, Voices};
