use drum_patterns::HitKind;

use super::*;
use crate::test_audio::peak;

const RATE: u32 = 48_000;

fn samples(preset: Preset, instrument: Instrument) -> Vec<f32> {
    preset_sound(preset, instrument, RATE)
        .hit(HitKind::Normal)
        .0
        .to_vec()
}

/// Sign changes a second: a rough pitch.
fn crossing_rate(s: &[f32]) -> f32 {
    let n = s
        .windows(2)
        .filter(|w| (w[0] < 0.0) != (w[1] < 0.0))
        .count();
    n as f32 / s.len() as f32 * RATE as f32
}

/// Milliseconds to the loudest sample.
fn to_peak_ms(s: &[f32]) -> f32 {
    let (at, _) =
        s.iter().enumerate().fold(
            (0, 0f32),
            |m, (i, x)| if x.abs() > m.1 { (i, x.abs()) } else { m },
        );
    at as f32 * 1000.0 / RATE as f32
}

#[test]
fn every_sound_peaks_at_full_scale_starts_on_the_hit_and_dies_away() {
    for preset in Preset::ALL {
        for instrument in Instrument::ALL {
            let s = samples(preset, instrument);
            let what = format!("{preset:?} {instrument:?}");
            assert!((peak(&s) - 1.0).abs() < 1e-4, "{what}");
            assert!(
                peak(&s[..frames(RATE as f32, 20.0)]) > 0.3,
                "{what} starts late"
            );
            assert!(peak(&s[s.len() - 48..]) < 0.01, "{what} stops on a click");
        }
    }
}

#[test]
fn the_presets_sound_different() {
    for instrument in Instrument::ALL {
        for (i, a) in Preset::ALL.iter().enumerate() {
            for b in &Preset::ALL[i + 1..] {
                assert_ne!(
                    samples(*a, instrument),
                    samples(*b, instrument),
                    "{instrument:?}: {a:?} and {b:?}"
                );
            }
        }
    }
}

#[test]
fn in_every_preset_the_crash_rings_longest_and_the_hi_hat_is_shortest() {
    for preset in Preset::ALL {
        let len = |i| samples(preset, i).len();
        assert!(len(Instrument::Crash) > len(Instrument::Kick), "{preset:?}");
        assert!(
            len(Instrument::HiHat) < len(Instrument::Snare),
            "{preset:?}"
        );
    }
}

#[test]
fn in_every_preset_the_kick_is_lowest_and_the_hi_hat_brightest() {
    for preset in Preset::ALL {
        let rate = |i| crossing_rate(&samples(preset, i));
        assert!(
            rate(Instrument::Kick) < rate(Instrument::Snare),
            "{preset:?}"
        );
        assert!(
            rate(Instrument::Snare) < rate(Instrument::HiHat),
            "{preset:?}"
        );
    }
}

#[test]
fn the_808_kick_booms_longer_than_the_909s() {
    let len = |p| samples(p, Instrument::Kick).len();
    assert!(len(Preset::Electro) > 2 * len(Preset::HipHop));
}

#[test]
fn jazz_brushes_swell_where_metal_hits_at_once() {
    let jazz = to_peak_ms(&samples(Preset::Jazz, Instrument::Snare));
    let metal = to_peak_ms(&samples(Preset::Metal, Instrument::Snare));
    assert!(jazz > 5.0, "jazz peaks at {jazz} ms");
    assert!(metal < 2.0, "metal peaks at {metal} ms");
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

/// Writes every preset's sounds, and a short groove in each style, as WAV
/// files into `DPG_RENDER_DIR` - to judge them by ear before the Sound
/// menu exists. Run: `cargo test -p dpg-samples render_presets --
/// --ignored`, with `DPG_RENDER_DIR` set to a folder under `samples/`.
#[test]
#[ignore = "writes WAV files into DPG_RENDER_DIR"]
fn render_presets() {
    let dir = std::env::var("DPG_RENDER_DIR").expect("set DPG_RENDER_DIR to a folder");
    std::fs::create_dir_all(&dir).unwrap();
    for preset in Preset::ALL {
        for instrument in Instrument::ALL {
            let s = samples(preset, instrument);
            write_wav(&format!("{dir}/{preset:?}-{instrument:?}.wav"), &s);
        }
        write_wav(&format!("{dir}/{preset:?}-groove.wav"), &groove(preset));
    }
}

/// Two bars at 100 BPM: 8ths on the hi-hat, the snare on 2 and 4, the kick
/// on 1 and the и of 2, the crash on the first 1.
fn groove(preset: Preset) -> Vec<f32> {
    let eighth = RATE as usize * 60 / 100 / 2;
    let mut out = vec![0f32; 16 * eighth + RATE as usize * 2];
    let mut add = |instrument, at: usize, gain: f32| {
        for (o, x) in out[at * eighth..]
            .iter_mut()
            .zip(samples(preset, instrument))
        {
            *o += x * gain;
        }
    };
    add(Instrument::Crash, 0, 0.35);
    for bar in [0, 8] {
        for e in 0..8 {
            add(Instrument::HiHat, bar + e, 0.3);
        }
        for e in [0, 3] {
            add(Instrument::Kick, bar + e, 0.7);
        }
        for e in [2, 6] {
            add(Instrument::Snare, bar + e, 0.55);
        }
    }
    let peak = peak(&out).max(1.0);
    out.iter().map(|x| x / peak).collect()
}

/// 16-bit mono PCM at `RATE`.
fn write_wav(path: &str, samples: &[f32]) {
    let data = (samples.len() * 2) as u32;
    let mut bytes = Vec::with_capacity(44 + data as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&RATE.to_le_bytes());
    bytes.extend_from_slice(&(RATE * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data.to_le_bytes());
    for x in samples {
        bytes.extend_from_slice(&((x.clamp(-1.0, 1.0) * 32_767.0) as i16).to_le_bytes());
    }
    std::fs::write(path, bytes).unwrap();
}
