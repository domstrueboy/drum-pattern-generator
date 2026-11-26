//! Plays grooves: what sounds when, how loud, on the audio thread. Knows
//! no device - it fills the blocks `dpg-hal` hands it.

mod clock;
mod engine;
mod metronome;
mod mix;
mod remote;
mod tap;
mod voices;

pub use clock::{BPM_MAX, BPM_MIN, Clock, Tick};
pub use engine::{DEFAULT_BPM, Engine};
pub use metronome::{CLICK_MS, Click, SUBDIVISION_DB, Subdivision, click_at};
pub use mix::{CHANNELS, Channel, MixControls, MixSettings, gains, taper};
pub use remote::{Remote, WrongRate};
pub use tap::{TAP_TIMEOUT, TapTempo};
pub use voices::{FADE_MS, SampleId, Trigger, VOICES, Voices};
