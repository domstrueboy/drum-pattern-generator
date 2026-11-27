use drum_patterns::HitKind;

use super::*;
use crate::test_audio::peak;

fn normal(kit: &Kit, instrument: Instrument) -> &[f32] {
    kit.sound(instrument).unwrap().hit(HitKind::Normal).0
}

#[test]
fn every_instrument_has_a_sound_at_full_scale() {
    let kit = placeholder_kit(48_000);
    assert_eq!(kit.rate(), 48_000);
    for instrument in Instrument::ALL {
        let samples = normal(&kit, instrument);
        assert!((peak(samples) - 1.0).abs() < 1e-6, "{instrument:?}");
    }
}

#[test]
fn each_sound_starts_on_the_hit() {
    let kit = placeholder_kit(48_000);
    for instrument in Instrument::ALL {
        let first_ms = &normal(&kit, instrument)[..48];
        assert!(peak(first_ms) > 0.3, "{instrument:?}");
    }
}

#[test]
fn the_crash_rings_longest_and_the_hi_hat_is_shortest() {
    let kit = placeholder_kit(44_100);
    let len = |i| normal(&kit, i).len();
    assert!(len(Instrument::Crash) > len(Instrument::Kick));
    assert!(len(Instrument::Kick) > len(Instrument::Snare));
    assert!(len(Instrument::Snare) > len(Instrument::HiHat));
}

#[test]
fn lengths_follow_the_rate() {
    let at = |rate| normal(&placeholder_kit(rate), Instrument::Kick).len();
    assert_eq!(at(96_000), 2 * at(48_000));
}

#[test]
fn the_same_rate_gives_the_same_kit() {
    assert_eq!(placeholder_kit(48_000), placeholder_kit(48_000));
}
