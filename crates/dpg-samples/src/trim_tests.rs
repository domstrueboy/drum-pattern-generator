use super::*;
use crate::decode::decode;
use crate::test_audio::peak;

const RATE: u32 = 44_100;
const MS: usize = 44; // frames in 1 ms at 44.1 kHz, rounded down

/// `before` ms of silence, a hit decaying over 100 ms from `level`, then
/// `after` ms of silence.
fn hit(before: usize, level: f32, after: usize) -> Audio {
    let decay = (0..100 * MS).map(|i| {
        let envelope = (-(i as f32) / (12.0 * MS as f32)).exp();
        let wave = (2.0 * std::f32::consts::PI * 200.0 * i as f32 / RATE as f32 + 1.0).sin();
        level * envelope * wave
    });
    let samples = std::iter::repeat_n(0.0, before * MS)
        .chain(decay)
        .chain(std::iter::repeat_n(0.0, after * MS))
        .collect();
    Audio {
        rate: RATE,
        samples,
    }
}

fn onset(audio: &Audio) -> usize {
    let p = peak(&audio.samples);
    audio
        .samples
        .iter()
        .position(|s| s.abs() >= p * ONSET)
        .unwrap()
}

#[test]
fn thirty_ms_in_front_go_and_the_hit_starts_on_time() {
    let trimmed = trim(hit(30, 0.8, 0)).unwrap();
    assert_eq!(onset(&trimmed), MS);
}

#[test]
fn the_attack_stays_whole() {
    let original = hit(30, 0.8, 0);
    let trimmed = trim(original.clone()).unwrap();
    // The first 10 ms of the hit, sample for sample.
    assert_eq!(
        trimmed.samples[MS..11 * MS],
        original.samples[30 * MS..40 * MS]
    );
}

#[test]
fn the_kept_millisecond_fades_in_from_silence() {
    let mut audio = hit(30, 0.8, 0);
    // A little hiss in front, 46 dB down.
    for (i, s) in audio.samples[..30 * MS].iter_mut().enumerate() {
        *s = if i % 2 == 0 { 0.004 } else { -0.004 };
    }
    let trimmed = trim(audio).unwrap();
    assert_eq!(onset(&trimmed), MS);
    assert_eq!(trimmed.samples[0], 0.0);
    assert!(trimmed.samples[..MS].iter().all(|s| s.abs() <= 0.004));
}

#[test]
fn a_hit_on_the_first_sample_loses_nothing() {
    let original = hit(0, 0.8, 0);
    let trimmed = trim(original.clone()).unwrap();
    assert_eq!(trimmed.samples[0], original.samples[0]);
}

#[test]
fn silence_after_the_ring_goes() {
    let trimmed = trim(hit(0, 0.8, 500)).unwrap();
    // The decay reaches -60 dB after about 83 ms.
    let ms = trimmed.samples.len() / MS;
    assert!((75..=100).contains(&ms), "{ms} ms");
    assert!(trimmed.samples.last().unwrap().abs() < 0.8 * 0.002);
}

#[test]
fn a_quiet_ghost_trims_like_a_loud_hit() {
    let loud = trim(hit(30, 0.8, 200)).unwrap();
    let quiet = trim(hit(30, 0.05, 200)).unwrap();
    assert_eq!(loud.samples.len(), quiet.samples.len());
    assert_eq!(onset(&quiet), MS);
}

#[test]
fn all_silence_is_nothing() {
    let silence = Audio {
        rate: RATE,
        samples: vec![0.0; 1000],
    };
    assert_eq!(trim(silence), None);
    assert_eq!(
        trim(Audio {
            rate: RATE,
            samples: vec![]
        }),
        None
    );
}

#[test]
fn a_one_sample_click_survives() {
    let trimmed = trim(Audio {
        rate: RATE,
        samples: vec![0.0, 0.0, 1.0, 0.0],
    })
    .unwrap();
    assert_eq!(trimmed.samples, [0.0, 0.0, 1.0]);
}

#[test]
fn compressed_tones_start_on_time() {
    // The tones are 0.5 s from their first sample. AAC pads the start
    // (the MP3 reader takes its padding off already) and rings ~20 ms
    // past the tone's hard stop, which is kept as part of the sound.
    for (name, ext) in [("tone-44k-stereo.mp3", "mp3"), ("tone-48k-mono.m4a", "m4a")] {
        let path = format!("{}/testdata/{name}", env!("CARGO_MANIFEST_DIR"));
        let audio = decode(&std::fs::read(path).unwrap(), Some(ext)).unwrap();
        let rate = audio.rate as f32;
        let padded = audio.samples.len() as f32 / rate;
        let trimmed = trim(audio).unwrap();
        let start = onset(&trimmed) as f32 / rate;
        let seconds = trimmed.samples.len() as f32 / rate;
        assert!(start < 0.002, "{name}: hit at {start} s");
        assert!(
            (0.49..0.53).contains(&seconds),
            "{name}: {padded} -> {seconds} s"
        );
    }
}
