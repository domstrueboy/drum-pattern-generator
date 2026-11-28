use dpg_engine::{DEFAULT_BPM, Engine};

use super::*;
use crate::names::{FeelName, InstrumentName, KindName, PulseName, SubdivisionName};

/// A session over an engine that is never played - the remote's settings
/// are what's under test.
fn session() -> (Session, Engine) {
    let (engine, remote) = Engine::new(48_000);
    (Session::new(remote, 1), engine)
}

fn apply(session: &mut Session, commands: &[Command]) -> Snapshot {
    for &command in commands {
        session.apply(command, 0.0);
    }
    session.snapshot()
}

#[test]
fn a_new_session_starts_as_the_spec_says() {
    let (session, _engine) = session();
    let s = session.snapshot();
    assert!(!s.playing);
    assert_eq!(s.bpm, DEFAULT_BPM);
    assert!(!s.metronome);
    assert_eq!(s.subdivision, SubdivisionName::Quarter);
    assert_eq!(s.bars, 1);
    assert_eq!(s.bar_skip, None);
    assert_eq!(s.crash_every, None);
    assert_eq!(s.hi_hat.pulse, PulseName::Eighth);
    assert_eq!(s.snare.feel, FeelName::Basic);
    assert_eq!(s.laps.len(), 1);
}

#[test]
fn the_same_seed_starts_the_same_groove() {
    let (a, _ea) = session();
    let (b, _eb) = session();
    assert_eq!(a.snapshot().laps, b.snapshot().laps);
}

#[test]
fn transport_and_tempo() {
    let (mut session, _engine) = session();
    let s = apply(
        &mut session,
        &[Command::StartStop, Command::NudgeBpm { by: 1.0 }],
    );
    assert!(s.playing);
    assert_eq!(s.bpm, DEFAULT_BPM + 1.0);
    let s = apply(
        &mut session,
        &[Command::StartStop, Command::Bpm { bpm: 999.0 }],
    );
    assert!(!s.playing);
    assert_eq!(s.bpm, 300.0);
}

#[test]
fn tap_tempo_uses_the_time_given() {
    let (mut session, _engine) = session();
    for now in [0.0, 0.5, 1.0] {
        session.apply(Command::Tap, now);
    }
    assert_eq!(session.snapshot().bpm, 120.0);
    for now in [10.0, 10.6, 11.2] {
        session.apply(Command::Tap, now);
    }
    assert_eq!(session.snapshot().bpm, 100.0);
}

#[test]
fn metronome_bar_skip_and_crash() {
    let (mut session, _engine) = session();
    let s = apply(
        &mut session,
        &[
            Command::Metronome,
            Command::Subdivision {
                value: SubdivisionName::Triplet,
            },
            Command::BarSkip { play: 2, skip: 1 },
            Command::Crash { every: Some(4) },
        ],
    );
    assert!(s.metronome);
    assert_eq!(s.subdivision, SubdivisionName::Triplet);
    assert_eq!(s.bar_skip, Some(BarSkipView { play: 2, skip: 1 }));
    assert_eq!(s.crash_every, Some(4));
    let s = apply(
        &mut session,
        &[Command::BarSkipOff, Command::Crash { every: None }],
    );
    assert_eq!(s.bar_skip, None);
    assert_eq!(s.crash_every, None);
}

#[test]
fn solo_and_mute_toggle() {
    let (mut session, _engine) = session();
    let snare = InstrumentName::Snare;
    let s = apply(
        &mut session,
        &[
            Command::Solo { instrument: snare },
            Command::Mute { instrument: snare },
        ],
    );
    assert!(s.mixer.snare.solo && s.mixer.snare.mute);
    assert!(!s.mixer.kick.solo);
    let s = apply(&mut session, &[Command::Solo { instrument: snare }]);
    assert!(!s.mixer.snare.solo);
}

#[test]
fn faders_move_their_own_channel() {
    let (mut session, _engine) = session();
    let s = apply(
        &mut session,
        &[
            Command::Fader {
                fader: Fader::Drums,
                position: 0.3,
            },
            Command::Fader {
                fader: Fader::Metronome,
                position: 0.4,
            },
            Command::Fader {
                fader: Fader::Instrument(InstrumentName::Kick),
                position: 0.5,
            },
        ],
    );
    assert_eq!(s.mixer.drums, 0.3);
    assert_eq!(s.mixer.metronome, 0.4);
    assert_eq!(s.mixer.kick.level, 0.5);
    assert_eq!(s.mixer.snare.level, 1.0);
}

#[test]
fn pattern_commands_change_the_laps() {
    let (mut session, _engine) = session();
    let s = apply(
        &mut session,
        &[
            Command::Bars { count: 2 },
            Command::Pulse {
                value: PulseName::DottedEighth,
            },
            Command::Feel {
                feel: FeelName::HalfTime,
            },
        ],
    );
    assert_eq!(s.bars, 2);
    assert_eq!(s.hi_hat.pulse, PulseName::DottedEighth);
    assert_eq!(s.laps.len(), 3);
    let snare: Vec<u32> = s.laps[0]
        .iter()
        .filter(|h| h.instrument == InstrumentName::Snare)
        .map(|h| h.tick)
        .collect();
    assert_eq!(snare, [24, 72]);
}

#[test]
fn ghost_notes_show_their_level_and_variant() {
    let (mut session, _engine) = session();
    let s = apply(
        &mut session,
        &[
            Command::Ghosts { count: 2 },
            Command::GenerateGhosts,
            Command::GhostRandomCount { count: 5 },
            Command::GhostRandom,
        ],
    );
    assert_eq!(s.snare.ghosts, 2);
    assert_eq!(s.snare.ghost_level, 2);
    assert_eq!(s.snare.ghost_random_count, 5);
    assert_eq!(s.snare.ghost_random_index, 1);
    let ghosts = s.laps[0].iter().filter(|h| h.kind == KindName::Ghost);
    assert_eq!(ghosts.count(), 2);
}

#[test]
fn kicks_report_their_counts() {
    let (mut session, _engine) = session();
    let s = apply(
        &mut session,
        &[
            Command::BaseKicks { count: 3 },
            Command::GenerateBaseKicks,
            Command::ExtraKicks { count: 1 },
            Command::GenerateExtraKicks,
        ],
    );
    assert_eq!(s.kick.base, 3);
    assert_eq!(s.kick.extra, 1);
    assert_eq!(s.kick.extra_level, 2);
}

#[test]
fn the_view_is_kept() {
    let (mut session, _engine) = session();
    let s = apply(
        &mut session,
        &[Command::View {
            view: View::Scrolled,
        }],
    );
    assert_eq!(s.view, View::Scrolled);
}

#[test]
fn the_snapshot_reads_as_the_ui_expects() {
    let (session, _engine) = session();
    let json = serde_json::to_value(session.snapshot()).unwrap();
    assert_eq!(json["hiHat"]["pulse"], "8");
    assert_eq!(json["snare"]["feel"], "basic");
    assert_eq!(json["ticksPerBar"], 48);
    assert_eq!(json["crashEvery"], serde_json::Value::Null);
    assert_eq!(json["laps"][0][0]["instrument"], "hiHat");
    let first = &json["notation"][0]["bars"][0][0];
    assert_eq!(first["events"][0]["label"], "8");
    assert_eq!(first["tuplet"], serde_json::Value::Null);
    assert_eq!(json["limits"]["variants"], 24);
    assert_eq!(json["limits"]["ghostLevels"], 4);
}

#[test]
fn each_lap_is_spelled() {
    let (mut session, _engine) = session();
    let s = apply(
        &mut session,
        &[Command::Pulse {
            value: PulseName::DottedEighth,
        }],
    );
    assert_eq!(s.notation.len(), s.laps.len());
    assert_ne!(s.notation[0], s.notation[1]);
}

#[test]
fn reconnecting_keeps_every_setting_but_stops() {
    let (mut session, _engine) = session();
    let s = apply(
        &mut session,
        &[
            Command::StartStop,
            Command::Bpm { bpm: 93.0 },
            Command::Metronome,
            Command::Subdivision {
                value: SubdivisionName::Sixteenth,
            },
            Command::BarSkip { play: 3, skip: 1 },
            Command::Crash { every: Some(8) },
            Command::Fader {
                fader: Fader::Drums,
                position: 0.2,
            },
            Command::Fader {
                fader: Fader::Instrument(InstrumentName::HiHat),
                position: 0.6,
            },
            Command::Solo {
                instrument: InstrumentName::Kick,
            },
            Command::Mute {
                instrument: InstrumentName::Snare,
            },
            Command::Ghosts { count: 2 },
        ],
    );
    let (_new_engine, remote) = Engine::new(44_100);
    session.reconnect(remote);
    let after = session.snapshot();
    assert!(!after.playing);
    assert_eq!(
        after,
        Snapshot {
            playing: false,
            ..s
        }
    );
}

#[test]
fn a_stopped_session_has_no_playhead() {
    let (mut session, _engine) = session();
    assert_eq!(session.playhead(0.0), None);
}
