//! The screen's state and every command, between the UI and the engine:
//! what happens when a button is pressed, testable without Tauri or a
//! device. The UI sends a [`Command`] and renders the [`Snapshot`] it gets
//! back.

mod command;
mod names;
mod parts;
mod session;
mod snapshot;

pub use command::{Command, Fader};
pub use names::{FeelName, InstrumentName, KindName, PulseName, SubdivisionName};
pub use parts::{Ladder, MAX_COUNT, Parts, RANDOM_COUNT, Set};
pub use session::{Session, View};
pub use snapshot::{
    BarSkipView, ChannelView, HiHatView, HitView, KickView, MixerView, Snapshot, SnareView,
};
