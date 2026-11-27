use super::*;

fn parse(line: &str) -> Command {
    Command::parse(line).unwrap()
}

#[test]
fn an_empty_line_is_a_new_groove() {
    assert_eq!(parse(""), Command::NewGroove);
    assert_eq!(parse("   "), Command::NewGroove);
}

#[test]
fn settings_take_their_values() {
    assert_eq!(parse("b 92"), Command::Bpm(92.0));
    assert_eq!(parse("m 8t"), Command::Subdivision(Subdivision::Triplet));
    assert_eq!(parse("f half"), Command::Feel(Feel::HalfTime));
    assert_eq!(parse("g 3 2"), Command::Ghosts(3, Some(2)));
    assert_eq!(parse("k 2"), Command::Kicks(2, None, None));
    assert_eq!(parse("k 2 1 3"), Command::Kicks(2, Some(1), Some(3)));
    assert_eq!(parse("h 8."), Command::HiHat(NoteValue::DottedEighth));
    assert_eq!(parse("n 4"), Command::Bars(4));
}

#[test]
fn random_defaults_to_half_and_turns_off() {
    assert_eq!(parse("r"), Command::Random(Some(50)));
    assert_eq!(parse("r 30"), Command::Random(Some(30)));
    assert_eq!(parse("r off"), Command::Random(None));
}

#[test]
fn bar_skip_and_crash_read_like_the_menus() {
    assert_eq!(parse("x 2-1"), Command::BarSkip(BarSkip::new(2, 1)));
    assert_eq!(parse("x off"), Command::BarSkip(None));
    assert_eq!(parse("c once"), Command::Crash(CrashRule::Once));
    assert_eq!(parse("c 8"), Command::Crash(CrashRule::Every(8)));
}

#[test]
fn the_mixer_names_its_faders() {
    assert_eq!(parse("v drums 0.6"), Command::Level(Fader::Drums, 0.6));
    assert_eq!(
        parse("v metronome 0.5"),
        Command::Level(Fader::Channel(Channel::Metronome), 0.5)
    );
    assert_eq!(parse("s kick"), Command::Solo(Channel::Kick));
    assert_eq!(parse("u hihat"), Command::Mute(Channel::HiHat));
}

#[test]
fn mistakes_say_what_is_wrong() {
    for line in [
        "z", "b", "b fast", "x 0-1", "x 21", "h 32", "m 3", "v bass 1",
    ] {
        assert!(Command::parse(line).is_err(), "{line}");
    }
}
