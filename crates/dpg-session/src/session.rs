//! The app's session: the parts, the engine's remote and the screen's own
//! choices together. A command changes them; a snapshot reports them.

use dpg_engine::{BarSkip, Channel, CrashRule, Remote};
use dpg_samples::{Kit, Preset, preset_sound};
use drum_patterns::{Instrument, Pattern};
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

use crate::command::{Command, Fader};
use crate::notation::notation;
use crate::parts::Parts;
use crate::playhead::{Glide, Playhead, locate};
use crate::snapshot::{
    BarSkipView, ChannelView, HiHatView, HitView, KickView, LIMITS, MixerView, Snapshot, SnareView,
    SoundsView,
};

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;

/// How the notation lays out several bars.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum View {
    /// All bars visible, one under another.
    #[default]
    Stacked,
    /// One row, scrolled left and right.
    Scrolled,
}

/// The screen's state, driving the engine.
pub struct Session {
    remote: Remote,
    parts: Parts,
    laps: Vec<Pattern>,
    view: View,
    rng: StdRng,
    glide: Glide,
    presets: Presets,
    kit: Kit,
}

/// The Sound menu's choices. The crash follows the hi-hat.
#[derive(Debug, Clone, Copy, Default)]
struct Presets {
    hi_hat: Preset,
    snare: Preset,
    kick: Preset,
}

impl Presets {
    fn of(&self, instrument: Instrument) -> Preset {
        match instrument {
            Instrument::HiHat | Instrument::Crash => self.hi_hat,
            Instrument::Snare => self.snare,
            Instrument::Kick => self.kick,
        }
    }

    /// The whole kit at `rate`. Made off the audio thread, then handed
    /// over whole.
    fn kit(&self, rate: u32) -> Kit {
        let mut kit = Kit::new(rate);
        for instrument in Instrument::ALL {
            kit.set(
                instrument,
                preset_sound(self.of(instrument), instrument, rate),
            );
        }
        kit
    }
}

impl Session {
    /// A first groove from `seed`, sent to the engine behind `remote`.
    /// The seed decides every random choice from here on.
    pub fn new(mut remote: Remote, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let parts = Parts::new(&mut rng);
        let laps = parts.build_laps();
        remote.set_laps(laps.clone());
        let presets = Presets::default();
        let kit = presets.kit(remote.rate());
        send_kit(&mut remote, &kit);
        Self {
            remote,
            parts,
            laps,
            view: View::default(),
            rng,
            glide: Glide::default(),
            presets,
            kit,
        }
    }

    /// The kit the engine plays, as the Sound menu chose it.
    pub fn kit(&self) -> &Kit {
        &self.kit
    }

    /// The engine's remote, for what the session doesn't cover: the kit,
    /// the playing position.
    pub fn remote(&mut self) -> &mut Remote {
        &mut self.remote
    }

    /// Moves the session to a new engine - after the output was lost to a
    /// phone call or a device change - and returns the old remote. Every
    /// setting and the pattern carry over; playing doesn't, so sound
    /// never comes back on its own.
    pub fn reconnect(&mut self, mut remote: Remote) -> Remote {
        let old = &self.remote;
        remote.set_bpm(old.bpm());
        remote.set_metronome(old.metronome());
        remote.set_subdivision(old.subdivision());
        remote.set_click_sound(old.click_sound());
        remote.set_bar_skip(old.bar_skip());
        remote.set_crash(old.crash());
        let mix = old.mix().snapshot();
        remote.mix().set_drums(mix.drums);
        for channel in Channel::ALL {
            remote.mix().set_level(channel, mix.levels[channel.index()]);
            remote.mix().set_solo(channel, bit(mix.solo, channel));
            remote.mix().set_mute(channel, bit(mix.mute, channel));
        }
        remote.set_laps(self.laps.clone());
        // The new output may run at another rate: the kit is made afresh.
        self.kit = self.presets.kit(remote.rate());
        send_kit(&mut remote, &self.kit);
        std::mem::replace(&mut self.remote, remote)
    }

    /// Carries out `command`. `now` is a steady clock in seconds, for Tap
    /// tempo.
    pub fn apply(&mut self, command: Command, now: f64) {
        self.remote.poll();
        let remote = &self.remote;
        let parts = &mut self.parts;
        let rng = &mut self.rng;
        let mix = remote.mix();
        match command {
            Command::StartStop if remote.is_playing() => remote.stop(),
            Command::StartStop => remote.play(),
            Command::Bpm { bpm } => remote.set_bpm(bpm),
            Command::NudgeBpm { by } => remote.set_bpm(remote.bpm() + by),
            Command::Tap => {
                self.remote.tap(now);
            }
            Command::Metronome => remote.set_metronome(!remote.metronome()),
            Command::MetronomeSound => remote.set_click_sound(remote.click_sound().next()),
            Command::SoundPreset { instrument, preset } => {
                self.set_preset(instrument.into(), preset.into());
            }
            Command::Subdivision { value } => remote.set_subdivision(value.into()),
            Command::View { view } => self.view = view,
            Command::BarSkip { play, skip } => remote.set_bar_skip(BarSkip::new(play, skip)),
            Command::BarSkipOff => remote.set_bar_skip(None),
            Command::Crash { every } => {
                remote.set_crash(every.map_or(CrashRule::Once, CrashRule::Every));
            }
            Command::Fader { fader, position } => match fader {
                Fader::Drums => mix.set_drums(position),
                Fader::Metronome => mix.set_level(Channel::Metronome, position),
                Fader::Instrument(i) => mix.set_level(channel(i.into()), position),
            },
            Command::Solo { instrument } => {
                let channel = channel(instrument.into());
                mix.set_solo(channel, !bit(mix.snapshot().solo, channel));
            }
            Command::Mute { instrument } => {
                let channel = channel(instrument.into());
                mix.set_mute(channel, !bit(mix.snapshot().mute, channel));
            }
            Command::Bars { count } => self.change(|p| p.set_bars(count)),
            Command::Pulse { value } => self.change(|p| p.set_pulse(value.into())),
            Command::HiHatRandom => self.change(Parts::hi_hat_random),
            Command::HiHatRandomCount { count } => {
                self.change(|p| p.set_hi_hat_random_count(count));
            }
            Command::Density { percent } => self.change(|p| p.set_density(percent)),
            Command::HiHatAccents { accents } => {
                self.change(|p| p.hi_hat_accents = accents.into());
            }
            Command::Feel { feel } => self.change(|p| p.set_feel(feel.into())),
            Command::Accents { count } => {
                parts.set_accents(count, rng);
                self.rebuild();
            }
            Command::Ghosts { count } => self.change(|p| p.set_ghosts(count)),
            Command::GenerateGhosts => {
                parts.generate_ghosts(rng);
                self.rebuild();
            }
            Command::GhostRandom => self.change(Parts::ghost_random),
            Command::GhostRandomCount { count } => {
                self.change(|p| p.set_ghost_random_count(count));
            }
            Command::BaseKicks { count } => self.change(|p| p.set_base_kicks(count)),
            Command::GenerateBaseKicks => {
                parts.generate_base_kicks(rng);
                self.rebuild();
            }
            Command::ExtraKicks { count } => self.change(|p| p.set_extra_kicks(count)),
            Command::GenerateExtraKicks => {
                parts.generate_extra_kicks(rng);
                self.rebuild();
            }
        }
    }

    /// What the screen shows now.
    pub fn snapshot(&self) -> Snapshot {
        let remote = &self.remote;
        let parts = &self.parts;
        let mix = remote.mix().snapshot();
        let channel = |c: Channel| ChannelView {
            level: mix.levels[c.index()],
            solo: bit(mix.solo, c),
            mute: bit(mix.mute, c),
        };
        Snapshot {
            playing: remote.is_playing(),
            bpm: remote.bpm(),
            metronome: remote.metronome(),
            subdivision: remote.subdivision().into(),
            metronome_sound: remote.click_sound().into(),
            sounds: SoundsView {
                hi_hat: self.presets.hi_hat.into(),
                snare: self.presets.snare.into(),
                kick: self.presets.kick.into(),
            },
            bars: parts.bars,
            view: self.view,
            bar_skip: remote.bar_skip().map(|s| BarSkipView {
                play: s.play(),
                skip: s.skip(),
            }),
            crash_every: match remote.crash() {
                CrashRule::Once => None,
                CrashRule::Every(bars) => Some(bars),
            },
            hi_hat: HiHatView {
                pulse: parts.pulse.into(),
                random: parts.hi_hat_random,
                random_count: parts.hi_hat_set.count(),
                random_index: parts.hi_hat_set.index(),
                density: parts.density,
                accents: parts.hi_hat_accents.into(),
            },
            snare: SnareView {
                feel: parts.feel.into(),
                accents: parts.accents,
                ghosts: parts.ghosts.count(),
                ghost_level: parts.ghosts.level(),
                ghost_random_count: parts.ghost_set.count(),
                ghost_random_index: parts.ghost_set.index(),
            },
            kick: KickView {
                base: parts.base_kicks,
                extra: parts.extra_kicks.count(),
                extra_level: parts.extra_kicks.level(),
            },
            mixer: MixerView {
                drums: mix.drums,
                metronome: mix.levels[Channel::Metronome.index()],
                hi_hat: channel(Channel::HiHat),
                snare: channel(Channel::Snare),
                kick: channel(Channel::Kick),
            },
            ticks_per_beat: parts.time.ticks_per_beat(),
            ticks_per_bar: parts.time.ticks_per_bar(),
            laps: self
                .laps
                .iter()
                .map(|lap| {
                    lap.hits()
                        .iter()
                        .map(|h| HitView {
                            tick: h.tick,
                            instrument: h.instrument.into(),
                            kind: h.kind.into(),
                        })
                        .collect()
                })
                .collect(),
            notation: self.laps.iter().map(notation).collect(),
            limits: LIMITS,
        }
    }

    /// Where the notation shows the sound at `now`, a steady clock in
    /// seconds; `None` while stopped. Asked once a screen frame.
    pub fn playhead(&mut self, now: f64) -> Option<Playhead> {
        let lap = &self.laps[0];
        let per_second = self.remote.bpm() / 60.0 * f64::from(lap.time().ticks_per_beat());
        let tick = self.glide.at(self.remote.position(), now, per_second)?;
        Some(locate(tick, lap.len_ticks(), self.laps.len()))
    }

    /// Gives `instrument` the sound of `preset` - the crash with the
    /// hi-hat - and sends the kit, also while playing. The crash alone is
    /// not a choice and is left as it is.
    fn set_preset(&mut self, instrument: Instrument, preset: Preset) {
        let changed: &[Instrument] = match instrument {
            Instrument::HiHat => {
                self.presets.hi_hat = preset;
                &[Instrument::HiHat, Instrument::Crash]
            }
            Instrument::Snare => {
                self.presets.snare = preset;
                &[Instrument::Snare]
            }
            Instrument::Kick => {
                self.presets.kick = preset;
                &[Instrument::Kick]
            }
            Instrument::Crash => return,
        };
        let rate = self.kit.rate();
        for &each in changed {
            self.kit.set(each, preset_sound(preset, each, rate));
        }
        send_kit(&mut self.remote, &self.kit);
    }

    fn change(&mut self, f: impl FnOnce(&mut Parts)) {
        f(&mut self.parts);
        self.rebuild();
    }

    /// Builds the laps again and sends them; the engine plays them from
    /// the same place in the bar.
    fn rebuild(&mut self) {
        self.laps = self.parts.build_laps();
        self.remote.set_laps(self.laps.clone());
    }
}

/// Sends a copy of `kit`; the session keeps its own to change and send
/// again.
fn send_kit(remote: &mut Remote, kit: &Kit) {
    remote
        .set_kit(kit.clone())
        .expect("every kit is made at the output's rate");
}

fn channel(instrument: Instrument) -> Channel {
    instrument.into()
}

fn bit(bits: u8, channel: Channel) -> bool {
    bits & 1 << channel.index() != 0
}
