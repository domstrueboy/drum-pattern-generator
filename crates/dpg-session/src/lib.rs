//! The screen's state and every command, between the UI and the engine:
//! what happens when a button is pressed, testable without Tauri or a
//! device. The UI sends a [`Command`] and renders the [`Snapshot`] it gets
//! back.

mod command;
mod names;
mod notation;
mod parts;
mod playhead;
mod session;
mod snapshot;

pub use command::{Command, Fader};
pub use names::{
    ClickSoundName, FeelName, HiHatAccentsName, InstrumentName, KindName, PresetName, PulseName,
    SubdivisionName,
};
pub use notation::{CountView, Event, Group, Notation, NoteHit, Tuplet, notation};
pub use parts::{Ladder, MAX_COUNT, Parts, RANDOM_COUNT, Set};
pub use playhead::{Glide, MAX_GLIDE, Playhead, locate};
pub use session::{Session, View};
pub use snapshot::{
    BarSkipView, ChannelView, HiHatView, HitView, KickView, LIMITS, Limits, MixerView, Snapshot,
    SnareView,
};
