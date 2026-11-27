use super::*;

fn parse(json: &str) -> Command {
    serde_json::from_str(json).unwrap()
}

#[test]
fn commands_read_from_the_ui_json() {
    assert_eq!(parse(r#"{"type":"startStop"}"#), Command::StartStop);
    assert_eq!(
        parse(r#"{"type":"nudgeBpm","by":-1}"#),
        Command::NudgeBpm { by: -1.0 }
    );
    assert_eq!(
        parse(r#"{"type":"pulse","value":"8."}"#),
        Command::Pulse {
            value: PulseName::DottedEighth
        }
    );
    assert_eq!(
        parse(r#"{"type":"barSkip","play":2,"skip":1}"#),
        Command::BarSkip { play: 2, skip: 1 }
    );
    assert_eq!(
        parse(r#"{"type":"crash","every":null}"#),
        Command::Crash { every: None }
    );
    assert_eq!(
        parse(r#"{"type":"hiHatRandomCount","count":8}"#),
        Command::HiHatRandomCount { count: 8 }
    );
    assert_eq!(
        parse(r#"{"type":"view","view":"scrolled"}"#),
        Command::View {
            view: View::Scrolled
        }
    );
}

#[test]
fn faders_name_their_channel() {
    let fader = |name: &str| match parse(&format!(
        r#"{{"type":"fader","fader":"{name}","position":0.5}}"#
    )) {
        Command::Fader { fader, .. } => fader,
        other => panic!("{other:?}"),
    };
    assert_eq!(fader("drums"), Fader::Drums);
    assert_eq!(fader("metronome"), Fader::Metronome);
    assert_eq!(fader("hiHat"), Fader::Instrument(InstrumentName::HiHat));
}

#[test]
fn an_unknown_command_is_an_error() {
    assert!(serde_json::from_str::<Command>(r#"{"type":"dance"}"#).is_err());
    assert!(serde_json::from_str::<Command>(r#"{"type":"pulse","value":"32"}"#).is_err());
}
