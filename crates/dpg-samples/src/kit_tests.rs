use std::io::Cursor;

use super::*;
use crate::test_audio::peak;

/// A WAV file of `before_ms` silence, then a 200 Hz hit at `level`
/// decaying over 100 ms.
fn hit_wav(rate: u32, before_ms: u32, level: f32) -> Vec<u8> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut bytes = Cursor::new(Vec::new());
    let mut writer = hound::WavWriter::new(&mut bytes, spec).unwrap();
    for _ in 0..rate * before_ms / 1000 {
        writer.write_sample(0.0f32).unwrap();
    }
    for i in 0..rate / 10 {
        let t = i as f32 / rate as f32;
        let s = level * (-t / 0.012).exp() * (2.0 * std::f32::consts::PI * 200.0 * t + 1.0).sin();
        writer.write_sample(s).unwrap();
    }
    writer.finalize().unwrap();
    bytes.into_inner()
}

fn wav(bytes: &[u8]) -> SampleFile<'_> {
    SampleFile {
        bytes,
        extension: Some("wav"),
    }
}

fn db(level: f32) -> f32 {
    20.0 * level.log10()
}

#[test]
fn one_sample_plays_every_kind_louder_or_quieter() {
    let sound = Sound::new(vec![1.0, 0.5], None);
    let (accent, normal, ghost) = (
        sound.hit(HitKind::Accent),
        sound.hit(HitKind::Normal),
        sound.hit(HitKind::Ghost),
    );
    assert!(accent.0 == normal.0 && normal.0 == ghost.0);
    assert!((db(accent.1) - ACCENT_DB).abs() < 0.01);
    assert!((db(normal.1) - NORMAL_DB).abs() < 0.01);
    assert!((db(ghost.1) - GHOST_DB).abs() < 0.01);
}

#[test]
fn two_samples_give_the_ghost_its_own() {
    let sound = Sound::new(vec![1.0], Some(vec![0.2]));
    assert_eq!(sound.hit(HitKind::Accent).0, [1.0]);
    assert_eq!(sound.hit(HitKind::Normal).0, [1.0]);
    let (ghost, ghost_gain) = sound.hit(HitKind::Ghost);
    assert_eq!(ghost, [0.2]);
    // Its own quietness is in the sample, so it plays at the normal gain.
    assert_eq!(ghost_gain, sound.hit(HitKind::Normal).1);
    assert!(sound.hit(HitKind::Accent).1 > ghost_gain);
}

#[test]
fn loading_brings_a_hit_to_the_rate_on_time_at_full_scale() {
    let file = hit_wav(44_100, 30, 0.3);
    let sound = Sound::load(wav(&file), None, 48_000).unwrap();
    let (samples, _) = sound.hit(HitKind::Normal);
    assert!((peak(samples) - 1.0).abs() < 1e-4, "peak {}", peak(samples));
    // 30 ms of silence gone: the hit starts 1 ms (48 frames) in.
    let onset = samples.iter().position(|s| s.abs() >= 0.01).unwrap();
    assert!((47..=49).contains(&onset), "onset {onset}");
    // About 100 ms of decay at 48 kHz, cut where it falls to -60 dB.
    assert!(
        (3_500..=4_900).contains(&samples.len()),
        "{}",
        samples.len()
    );
}

#[test]
fn a_ghost_sample_keeps_its_recorded_quietness() {
    let (normal, ghost) = (hit_wav(48_000, 0, 0.5), hit_wav(48_000, 0, 0.1));
    let sound = Sound::load(wav(&normal), Some(wav(&ghost)), 48_000).unwrap();
    let ghost_peak = peak(sound.hit(HitKind::Ghost).0);
    assert!((ghost_peak - 0.2).abs() < 1e-3, "ghost peak {ghost_peak}");
}

#[test]
fn errors_say_which_step_failed() {
    let garbage = [7u8; 512];
    let silence = hit_wav(48_000, 50, 0.0);
    let file = hit_wav(48_000, 0, 0.5);
    assert!(matches!(
        Sound::load(wav(&garbage), None, 48_000),
        Err(LoadError::Decode(_))
    ));
    assert_eq!(
        Sound::load(wav(&silence), None, 48_000),
        Err(LoadError::Silent)
    );
    assert!(matches!(
        Sound::load(wav(&file), Some(wav(&garbage)), 48_000),
        Err(LoadError::Decode(_))
    ));
    assert!(matches!(
        Sound::load(wav(&file), None, 0),
        Err(LoadError::Resample(_))
    ));
}

#[test]
fn a_kit_holds_a_sound_per_instrument() {
    let mut kit = Kit::new(48_000);
    assert_eq!(kit.rate(), 48_000);
    assert!(Instrument::ALL.iter().all(|&i| kit.sound(i).is_none()));

    kit.set(Instrument::Snare, Sound::new(vec![0.5], None));
    kit.set(Instrument::Kick, Sound::new(vec![0.9], None));
    kit.set(Instrument::Snare, Sound::new(vec![1.0], None));

    assert_eq!(
        kit.sound(Instrument::Snare).unwrap().hit(HitKind::Accent).0,
        [1.0]
    );
    assert_eq!(
        kit.sound(Instrument::Kick).unwrap().hit(HitKind::Accent).0,
        [0.9]
    );
    assert!(kit.sound(Instrument::HiHat).is_none());
}

/// Loads a real kit from `DPG_KIT_DIR` (the DP-2000 WAV export) and prints
/// what the chain did to each file. Local only - the files never enter the
/// repo. Run: `cargo test -p dpg-samples real_kit -- --ignored --nocapture`.
#[test]
#[ignore = "needs a local kit in DPG_KIT_DIR"]
fn real_kit_loads() {
    let dir = std::env::var("DPG_KIT_DIR").expect("set DPG_KIT_DIR to the kit's folder");
    let read = |name: &str| std::fs::read(format!("{dir}/{name}")).unwrap();
    let rate = 48_000;
    let parts = [
        (Instrument::Crash, "28_CRSH18_H.wav", None),
        (Instrument::HiHat, "17_HH_CL_H.wav", None),
        (Instrument::Snare, "03_SNR_H.wav", Some("06_SNR_GHST.wav")),
        (Instrument::Kick, "01_KICK_H.wav", None),
    ];
    let mut kit = Kit::new(rate);
    for (instrument, normal, ghost) in parts {
        let normal_bytes = read(normal);
        let ghost_bytes = ghost.map(read);
        let sound = Sound::load(wav(&normal_bytes), ghost_bytes.as_deref().map(wav), rate).unwrap();

        for (name, bytes, kind) in [
            (Some(normal), Some(&normal_bytes), HitKind::Normal),
            (ghost, ghost_bytes.as_ref(), HitKind::Ghost),
        ] {
            let (Some(name), Some(bytes)) = (name, bytes) else {
                continue;
            };
            let raw = resample(decode(bytes, Some("wav")).unwrap(), rate).unwrap();
            let raw_peak = peak(&raw.samples);
            let lead = raw
                .samples
                .iter()
                .position(|s| s.abs() >= raw_peak * 0.01)
                .unwrap();
            let (samples, _) = sound.hit(kind);
            let ms = |frames: usize| frames as f32 * 1000.0 / rate as f32;
            println!(
                "{instrument:?} {name}: {:.0} ms -> {:.0} ms, {:.1} ms cut in front, peak {:.1} dB",
                ms(raw.samples.len()),
                ms(samples.len()),
                ms(lead.saturating_sub(48)),
                db(peak(samples)),
            );
            let onset = samples
                .iter()
                .position(|s| s.abs() >= peak(samples) * 0.01)
                .unwrap();
            assert!(onset <= 48, "{name}: hit starts {onset} frames in");
        }
        kit.set(instrument, sound);
    }
    let snare = kit.sound(Instrument::Snare).unwrap();
    assert!(peak(snare.hit(HitKind::Ghost).0) < peak(snare.hit(HitKind::Normal).0));
}
