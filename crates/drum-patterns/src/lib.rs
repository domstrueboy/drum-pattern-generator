//! Drum patterns and the rules that generate them. Pure: no audio, no UI,
//! no I/O, so both this app and looper-pedal's drum machine can use it.

mod count;
mod hi_hat;
mod konnakol;
mod pattern;
mod snare;
mod time;

pub use count::{CountLabel, CountMark, count};
pub use hi_hat::hi_hat_pulse;
pub use konnakol::{KonnakolMark, Syllable, konnakol};
pub use pattern::{Hit, HitKind, Instrument, MAX_BARS, Pattern, PatternError};
pub use snare::{Feel, snare};
pub use time::{NoteValue, TICKS_PER_QUARTER, TimeSignature};
