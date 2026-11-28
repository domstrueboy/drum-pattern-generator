//! What the screen shows, sent whole after every command. Names follow
//! the UI's (camelCase), values the spec's menus.

use serde::Serialize;

use crate::names::{
    ClickSoundName, FeelName, HiHatAccentsName, InstrumentName, KindName, PresetName, PulseName,
    SubdivisionName,
};
use crate::notation::Notation;
use crate::session::View;

/// The whole screen's state.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    /// Whether the groove plays.
    pub playing: bool,
    /// Beats a minute.
    pub bpm: f64,
    /// Whether the metronome clicks.
    pub metronome: bool,
    /// The metronome's clicks a beat.
    pub subdivision: SubdivisionName,
    /// The metronome's sound.
    pub metronome_sound: ClickSoundName,
    /// The Sound menu's choices.
    pub sounds: SoundsView,
    /// The pattern's length in bars.
    pub bars: u8,
    /// Stacked or scrolled bars.
    pub view: View,
    /// Bar skip, when on.
    pub bar_skip: Option<BarSkipView>,
    /// The crash every this many bars; `None` is once at the start.
    pub crash_every: Option<u8>,
    /// The Hi-hat submenu.
    pub hi_hat: HiHatView,
    /// The Snare submenu.
    pub snare: SnareView,
    /// The Kick submenu.
    pub kick: KickView,
    /// The faders, solos and mutes.
    pub mixer: MixerView,
    /// Ticks a beat (12 in 4/4).
    pub ticks_per_beat: u32,
    /// Ticks a bar (48 in 4/4).
    pub ticks_per_bar: u32,
    /// The pattern on each time through: one lap for most grooves, up to
    /// three while a dotted hi-hat drifts.
    pub laps: Vec<Vec<HitView>>,
    /// Each lap spelled as notation, ready to draw.
    pub notation: Vec<Notation>,
    /// How far the menus' counts and levels go.
    pub limits: Limits,
}

/// The menus' ranges, so the UI never keeps its own copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Limits {
    /// Most ghost notes, accents or kicks a bar.
    pub count: u8,
    /// Most variants Random holds.
    pub variants: u8,
    /// Ghost notes' top level.
    pub ghost_levels: u8,
    /// Extra kicks' top level.
    pub extra_kick_levels: u8,
}

/// The ranges `dpg-session` and `drum-patterns` keep to.
pub const LIMITS: Limits = Limits {
    count: crate::parts::MAX_COUNT,
    variants: drum_patterns::Variants::MAX,
    ghost_levels: drum_patterns::GHOST_LEVELS,
    extra_kick_levels: drum_patterns::EXTRA_KICK_LEVELS,
};

/// Each instrument's kit style; the crash follows the hi-hat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SoundsView {
    /// The hi-hat's, and the crash's.
    pub hi_hat: PresetName,
    /// The snare's.
    pub snare: PresetName,
    /// The kick's.
    pub kick: PresetName,
}

/// Bar skip's choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BarSkipView {
    /// Bars heard.
    pub play: u8,
    /// Bars silent.
    pub skip: u8,
}

/// The Hi-hat submenu's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HiHatView {
    /// The pulse.
    pub pulse: PulseName,
    /// Whether Random plays instead of the full pulse.
    pub random: bool,
    /// Random's variants.
    pub random_count: u8,
    /// Random's variant playing, from 0.
    pub random_index: u8,
    /// Off-beat hits Random keeps, in percent.
    pub density: u8,
    /// Where the hits are accented.
    pub accents: HiHatAccentsName,
}

/// The Snare submenu's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnareView {
    /// The time feel.
    pub feel: FeelName,
    /// Accents a bar.
    pub accents: u8,
    /// Ghost notes a bar.
    pub ghosts: u8,
    /// The ghost notes' level, from 1.
    pub ghost_level: u8,
    /// The ghost notes' Random variants.
    pub ghost_random_count: u8,
    /// The ghost notes' variant playing, from 0.
    pub ghost_random_index: u8,
}

/// The Kick submenu's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KickView {
    /// Base kicks a bar.
    pub base: u8,
    /// Extra kicks a bar.
    pub extra: u8,
    /// The extra kicks' level, from 1.
    pub extra_level: u8,
}

/// The faders, solos and mutes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MixerView {
    /// The Drums fader, 0 to 1.
    pub drums: f32,
    /// The Metronome fader, 0 to 1.
    pub metronome: f32,
    /// The hi-hat's fader and buttons.
    pub hi_hat: ChannelView,
    /// The snare's.
    pub snare: ChannelView,
    /// The kick's.
    pub kick: ChannelView,
}

/// One instrument's fader, S and M.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ChannelView {
    /// 0 to 1.
    pub level: f32,
    /// Solo on.
    pub solo: bool,
    /// Mute on.
    pub mute: bool,
}

/// One hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct HitView {
    /// From the pattern's start, 12 a quarter note.
    pub tick: u32,
    /// What is hit.
    pub instrument: InstrumentName,
    /// How hard.
    pub kind: KindName,
}
