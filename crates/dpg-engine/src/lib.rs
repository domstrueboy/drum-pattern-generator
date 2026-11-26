//! Plays grooves: what sounds when, how loud, on the audio thread. Knows
//! no device - it fills the blocks `dpg-hal` hands it.

mod clock;

pub use clock::{BPM_MAX, BPM_MIN, Clock, Tick};
