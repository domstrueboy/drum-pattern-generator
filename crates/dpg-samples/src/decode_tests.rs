use std::io::Cursor;

use super::*;

const TONE_HZ: f32 = 440.0;

/// A WAV file in memory, one `frames` entry per frame, one value per channel.
fn wav(rate: u32, bits: u16, float: bool, frames: &[Vec<f32>]) -> Vec<u8> {
    let spec = hound::WavSpec {
        channels: frames[0].len() as u16,
        sample_rate: rate,
        bits_per_sample: bits,
        sample_format: if float {
            hound::SampleFormat::Float
        } else {
            hound::SampleFormat::Int
        },
    };
    let mut bytes = Cursor::new(Vec::new());
    let mut writer = hound::WavWriter::new(&mut bytes, spec).unwrap();
    let full = ((1i64 << (bits - 1)) - 1) as f32;
    for frame in frames {
        for &s in frame {
            if float {
                writer.write_sample(s).unwrap();
            } else {
                writer.write_sample((s * full).round() as i32).unwrap();
            }
        }
    }
    writer.finalize().unwrap();
    bytes.into_inner()
}

fn tone(rate: u32, seconds: f32, channels: usize) -> Vec<Vec<f32>> {
    let frames = (rate as f32 * seconds) as usize;
    (0..frames)
        .map(|i| {
            let s = 0.5 * (2.0 * std::f32::consts::PI * TONE_HZ * i as f32 / rate as f32).sin();
            vec![s; channels]
        })
        .collect()
}

fn fixture(name: &str) -> Vec<u8> {
    let path = format!("{}/testdata/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read(path).unwrap()
}

fn peak(samples: &[f32]) -> f32 {
    samples.iter().fold(0.0, |m, s| m.max(s.abs()))
}

/// The tone's pitch from its rising zero crossings, over the middle half
/// (lossy codecs smear the edges).
fn pitch(audio: &Audio) -> f32 {
    let n = audio.samples.len();
    let middle = &audio.samples[n / 4..3 * n / 4];
    let rising = middle
        .windows(2)
        .filter(|w| w[0] < 0.0 && w[1] >= 0.0)
        .count();
    rising as f32 * audio.rate as f32 / middle.len() as f32
}

/// Checks a decoded 0.5 s half-scale tone at 440 Hz.
fn assert_tone(audio: &Audio, rate: u32) {
    assert_eq!(audio.rate, rate);
    let seconds = audio.samples.len() as f32 / rate as f32;
    // Lossy codecs pad the start and end a little, and overshoot where
    // the tone starts and stops abruptly.
    assert!((0.49..0.6).contains(&seconds), "{seconds} s");
    assert!(
        (0.45..0.65).contains(&peak(&audio.samples)),
        "peak {}",
        peak(&audio.samples)
    );
    assert!(
        (TONE_HZ - 5.0..TONE_HZ + 5.0).contains(&pitch(audio)),
        "{} Hz",
        pitch(audio)
    );
}

#[test]
fn reads_a_16_bit_wav() {
    let audio = decode(&wav(44_100, 16, false, &tone(44_100, 0.5, 1)), Some("wav")).unwrap();
    assert_eq!(audio.samples.len(), 22_050);
    assert_tone(&audio, 44_100);
}

#[test]
fn reads_a_24_bit_wav_without_a_hint() {
    let audio = decode(&wav(48_000, 24, false, &tone(48_000, 0.5, 1)), None).unwrap();
    assert_tone(&audio, 48_000);
}

#[test]
fn reads_a_float_wav() {
    let audio = decode(&wav(44_100, 32, true, &tone(44_100, 0.5, 1)), Some("wav")).unwrap();
    assert_tone(&audio, 44_100);
}

#[test]
fn stereo_becomes_the_average_of_both_sides() {
    let frames = vec![vec![0.5, -0.1]; 100];
    let audio = decode(&wav(44_100, 24, false, &frames), Some("wav")).unwrap();
    assert_eq!(audio.samples.len(), 100);
    assert!(
        audio.samples.iter().all(|s| (s - 0.2).abs() < 1e-4),
        "{:?}",
        &audio.samples[..4]
    );
}

#[test]
fn reads_flac() {
    assert_tone(
        &decode(&fixture("tone-44k-mono.flac"), Some("flac")).unwrap(),
        44_100,
    );
}

#[test]
fn reads_stereo_mp3() {
    assert_tone(
        &decode(&fixture("tone-44k-stereo.mp3"), Some("mp3")).unwrap(),
        44_100,
    );
}

#[test]
fn reads_m4a_as_phones_record_it() {
    assert_tone(
        &decode(&fixture("tone-48k-mono.m4a"), Some("m4a")).unwrap(),
        48_000,
    );
}

#[test]
fn a_wrong_hint_is_overruled_by_the_contents() {
    assert_tone(
        &decode(&fixture("tone-44k-mono.flac"), Some("mp3")).unwrap(),
        44_100,
    );
}

#[test]
fn garbage_is_an_error_not_a_crash() {
    let garbage: Vec<u8> = (0..4096u32).map(|i| (i * 7919 % 251) as u8).collect();
    assert!(decode(&garbage, None).is_err());
    assert!(decode(&[], Some("wav")).is_err());
}

#[test]
fn a_cut_off_file_keeps_what_it_has() {
    let bytes = fixture("tone-44k-stereo.mp3");
    let audio = decode(&bytes[..bytes.len() / 2], Some("mp3")).unwrap();
    assert!(audio.samples.len() > 4_000, "{}", audio.samples.len());
}

#[test]
fn a_wav_with_no_frames_is_empty() {
    let bytes = wav(44_100, 16, false, &[vec![0.0]])[..44].to_vec();
    assert_eq!(decode(&bytes, Some("wav")), Err(DecodeError::Empty));
}

#[test]
fn a_whole_song_is_too_long() {
    let frames = vec![vec![0.0]; 8_000 * (MAX_SECONDS as usize + 1)];
    let audio = decode(&wav(8_000, 16, false, &frames), Some("wav"));
    assert_eq!(audio, Err(DecodeError::TooLong));
}
