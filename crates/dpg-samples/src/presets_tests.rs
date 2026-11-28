use drum_patterns::HitKind;

use super::*;
use crate::test_audio::peak;

#[test]
fn every_preset_has_every_instrument_at_full_scale() {
    for preset in Preset::ALL {
        for instrument in Instrument::ALL {
            let sound = preset_sound(preset, instrument, 48_000);
            let samples = sound.hit(HitKind::Normal).0;
            assert!(!samples.is_empty(), "{preset:?} {instrument:?}");
            assert!(
                (peak(samples) - 1.0).abs() < 1e-6,
                "{preset:?} {instrument:?}"
            );
        }
    }
}

#[test]
fn the_same_preset_makes_the_same_sound() {
    assert_eq!(
        preset_sound(Preset::Electro, Instrument::Kick, 44_100),
        preset_sound(Preset::Electro, Instrument::Kick, 44_100)
    );
}

#[test]
fn a_sound_is_made_at_the_rate_asked() {
    let len = |rate| {
        preset_sound(Preset::Rock, Instrument::Kick, rate)
            .hit(HitKind::Normal)
            .0
            .len()
    };
    assert_eq!(len(96_000), 2 * len(48_000));
}
