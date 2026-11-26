use super::*;

fn settings() -> MixSettings {
    MixSettings {
        levels: [1.0; CHANNELS],
        drums: 1.0,
        solo: 0,
        mute: 0,
    }
}

fn gain(s: &MixSettings, channel: Channel) -> f32 {
    gains(s)[channel.index()]
}

#[test]
fn the_taper_is_squared_and_kept_in_range() {
    assert_eq!(taper(0.0), 0.0);
    assert_eq!(taper(0.5), 0.25);
    assert_eq!(taper(1.0), 1.0);
    assert_eq!(taper(1.5), 1.0);
    assert_eq!(taper(-1.0), 0.0);
}

#[test]
fn faders_up_play_everything_at_full() {
    assert_eq!(gains(&settings()), [1.0; CHANNELS]);
}

#[test]
fn the_drums_fader_moves_the_drums_not_the_metronome() {
    let s = MixSettings {
        drums: 0.5,
        ..settings()
    };
    for channel in [
        Channel::Crash,
        Channel::HiHat,
        Channel::Snare,
        Channel::Kick,
    ] {
        assert_eq!(gain(&s, channel), 0.25, "{channel:?}");
    }
    assert_eq!(gain(&s, Channel::Metronome), 1.0);
}

#[test]
fn a_channel_fader_and_the_drums_fader_multiply() {
    let mut s = MixSettings {
        drums: 0.5,
        ..settings()
    };
    s.levels[Channel::Snare.index()] = 0.5;
    assert_eq!(gain(&s, Channel::Snare), 0.0625);
    assert_eq!(gain(&s, Channel::Kick), 0.25);
}

#[test]
fn mute_silences_one_channel() {
    let s = MixSettings {
        mute: Channel::Snare.bit(),
        ..settings()
    };
    assert_eq!(gains(&s), [1.0, 1.0, 0.0, 1.0, 1.0]);
}

#[test]
fn solo_leaves_only_the_soloed_and_the_metronome() {
    let s = MixSettings {
        solo: Channel::Kick.bit() | Channel::HiHat.bit(),
        ..settings()
    };
    // The crash has no solo button: a solo silences it.
    assert_eq!(gains(&s), [0.0, 1.0, 0.0, 1.0, 1.0]);
}

#[test]
fn mute_beats_solo() {
    let s = MixSettings {
        solo: Channel::Kick.bit(),
        mute: Channel::Kick.bit(),
        ..settings()
    };
    assert_eq!(gain(&s, Channel::Kick), 0.0);
}

#[test]
fn the_controls_hand_over_what_the_ui_set() {
    let controls = MixControls::default();
    assert_eq!(controls.snapshot(), MixSettings::DEFAULT);

    controls.set_level(Channel::HiHat, 0.3);
    controls.set_drums(2.0);
    controls.set_solo(Channel::Snare, true);
    controls.set_mute(Channel::Kick, true);
    controls.set_mute(Channel::Crash, true);
    controls.set_mute(Channel::Crash, false);

    let s = controls.snapshot();
    assert_eq!(s.levels[Channel::HiHat.index()], 0.3);
    assert_eq!(s.drums, 1.0);
    assert_eq!(s.solo, Channel::Snare.bit());
    assert_eq!(s.mute, Channel::Kick.bit());
}

#[test]
fn every_instrument_has_its_channel() {
    use drum_patterns::Instrument;
    let channels: Vec<Channel> = Instrument::ALL.iter().map(|&i| i.into()).collect();
    assert_eq!(channels, Channel::ALL[..4]);
}
