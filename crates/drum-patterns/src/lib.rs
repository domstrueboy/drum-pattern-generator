//! Drum patterns and the rules that generate them. Pure: no audio, no UI,
//! no I/O, so both this app and looper-pedal's drum machine can use it.

mod count;
mod ghost_notes;
mod hi_hat;
mod kick;
mod konnakol;
mod ladder;
mod pattern;
mod snare;
mod time;

pub use count::{CountLabel, CountMark, count};
pub use ghost_notes::{GHOST_LEVELS, ghost_notes};
pub use hi_hat::hi_hat_pulse;
pub use kick::{EXTRA_KICK_LEVELS, base_kicks, extra_kicks};
pub use konnakol::{KonnakolMark, Syllable, konnakol};
pub use pattern::{Hit, HitKind, Instrument, MAX_BARS, Pattern, PatternError};
pub use snare::{Feel, snare};
pub use time::{NoteValue, TICKS_PER_QUARTER, TimeSignature};
