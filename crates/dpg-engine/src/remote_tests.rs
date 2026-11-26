use super::*;
use crate::Engine;

#[test]
fn the_tempo_is_clamped_to_the_range() {
    let (_engine, remote) = Engine::new(48_000);
    remote.set_bpm(1_000.0);
    assert_eq!(remote.bpm(), BPM_MAX);
    remote.set_bpm(5.0);
    assert_eq!(remote.bpm(), BPM_MIN);
}

#[test]
fn a_kit_at_another_rate_is_refused() {
    let (_engine, mut remote) = Engine::new(48_000);
    let refused = remote.set_kit(Kit::new(44_100));
    assert_eq!(
        refused,
        Err(WrongRate {
            kit: 44_100,
            output: 48_000
        })
    );
}

#[test]
fn start_and_stop_are_remembered() {
    let (_engine, remote) = Engine::new(48_000);
    assert!(!remote.is_playing());
    remote.play();
    assert!(remote.is_playing());
    remote.stop();
    assert!(!remote.is_playing());
    assert_eq!(remote.position(), None, "nothing rendered yet");
}

#[test]
fn tap_tempo_sets_the_tempo() {
    let (_engine, mut remote) = Engine::new(48_000);
    assert_eq!(remote.tap(0.0), None);
    assert_eq!(remote.tap(0.6), Some(100.0));
    assert_eq!(remote.bpm(), 100.0);
}

#[test]
fn the_metronome_starts_off_on_quarters() {
    let (_engine, remote) = Engine::new(48_000);
    assert!(!remote.metronome());
    assert_eq!(remote.subdivision(), Subdivision::Quarter);
    remote.set_metronome(true);
    remote.set_subdivision(Subdivision::Triplet);
    assert!(remote.metronome());
    assert_eq!(remote.subdivision(), Subdivision::Triplet);
}
