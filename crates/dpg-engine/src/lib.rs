//! Plays grooves: what sounds when, how loud, on the audio thread. Knows
//! no device - it fills the blocks `dpg-hal` hands it.

mod clock;
mod engine;
mod mix;
mod remote;
mod voices;

pub use clock::{BPM_MAX, BPM_MIN, Clock, Tick};
pub use engine::{DEFAULT_BPM, Engine};
pub use mix::{CHANNELS, Channel, MixControls, MixSettings, gains, taper};
pub use remote::{Remote, WrongRate};
pub use voices::{FADE_MS, SampleId, Trigger, VOICES, Voices};
