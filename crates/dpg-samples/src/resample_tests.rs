use super::*;
use crate::test_audio::{TONE_HZ, peak, pitch, sine};

fn tone(rate: u32) -> Audio {
    Audio {
        rate,
        samples: sine(rate, 0.5, TONE_HZ),
    }
}

/// One full-scale sample at `at`, silence around it.
fn click(rate: u32, frames: usize, at: usize) -> Audio {
    let mut samples = vec![0.0; frames];
    samples[at] = 1.0;
    Audio { rate, samples }
}

fn loudest_frame(samples: &[f32]) -> usize {
    (0..samples.len())
        .max_by(|&a, &b| samples[a].abs().total_cmp(&samples[b].abs()))
        .unwrap()
}

#[test]
fn the_same_rate_passes_through_untouched() {
    assert_eq!(resample(tone(48_000), 48_000), Ok(tone(48_000)));
}

// Pitch is measured to about +/-4 Hz; a wrong rate would be 39 Hz off.
#[test]
fn up_to_48k_keeps_pitch_level_and_length() {
    let audio = resample(tone(44_100), 48_000).unwrap();
    assert_eq!(audio.rate, 48_000);
    // 0.5 s, give or take the frame rounding adds.
    assert!(
        (24_000..=24_001).contains(&audio.samples.len()),
        "{}",
        audio.samples.len()
    );
    assert!(
        (TONE_HZ - 5.0..TONE_HZ + 5.0).contains(&pitch(&audio)),
        "{} Hz",
        pitch(&audio)
    );
    assert!(
        (0.48..0.53).contains(&peak(&audio.samples)),
        "peak {}",
        peak(&audio.samples)
    );
}

#[test]
fn down_to_44k_keeps_pitch_level_and_length() {
    let audio = resample(tone(48_000), 44_100).unwrap();
    assert!(
        (22_050..=22_051).contains(&audio.samples.len()),
        "{}",
        audio.samples.len()
    );
    assert!(
        (TONE_HZ - 5.0..TONE_HZ + 5.0).contains(&pitch(&audio)),
        "{} Hz",
        pitch(&audio)
    );
    assert!(
        (0.48..0.53).contains(&peak(&audio.samples)),
        "peak {}",
        peak(&audio.samples)
    );
}

#[test]
fn a_hit_stays_on_time() {
    // At 44.1 kHz frame 4410 is 100 ms in; at 48 kHz that is frame 4800.
    let audio = resample(click(44_100, 8_820, 4_410), 48_000).unwrap();
    let at = loudest_frame(&audio.samples);
    assert!((4_799..=4_801).contains(&at), "frame {at}");
}

#[test]
fn a_hit_at_the_very_start_survives() {
    let audio = resample(click(44_100, 4_410, 0), 48_000).unwrap();
    assert!(loudest_frame(&audio.samples) <= 1);
    assert!(peak(&audio.samples) > 0.5);
}

#[test]
fn what_the_new_rate_cannot_hold_is_filtered_out() {
    // 30 kHz fits at 96 kHz but not at 44.1 kHz, whose limit is 22 kHz;
    // unfiltered it would fold back as an audible 14 kHz whine.
    let high = Audio {
        rate: 96_000,
        samples: sine(96_000, 0.5, 30_000.0),
    };
    let audio = resample(high, 44_100).unwrap();
    // Unfiltered it would stay near 0.5. The middle only: the tone's hard
    // start and stop are clicks, which do reach into what 44.1 kHz holds.
    let n = audio.samples.len();
    let middle = peak(&audio.samples[n / 4..3 * n / 4]);
    assert!(middle < 0.005, "peak {middle}");
}

#[test]
fn a_tiny_sound_still_resamples() {
    let audio = resample(click(44_100, 10, 3), 48_000).unwrap();
    assert!(
        (10..=12).contains(&audio.samples.len()),
        "{}",
        audio.samples.len()
    );
}

#[test]
fn a_zero_rate_is_an_error() {
    assert!(resample(tone(44_100), 0).is_err());
}
