use super::*;
use crate::test_audio::{TONE_HZ, peak, sine};

fn tone(level: f32) -> Audio {
    let samples = sine(44_100, 0.1, TONE_HZ)
        .iter()
        .map(|s| s * 2.0 * level)
        .collect();
    Audio {
        rate: 44_100,
        samples,
    }
}

#[test]
fn one_sample_reaches_full_scale() {
    let mut sound = tone(0.3);
    level([&mut sound]);
    assert!((peak(&sound.samples) - 1.0).abs() < 1e-6);
}

#[test]
fn a_ghost_stays_as_much_quieter_as_it_was_recorded() {
    let (mut normal, mut ghost) = (tone(0.5), tone(0.1));
    level([&mut normal, &mut ghost]);
    assert!((peak(&normal.samples) - 1.0).abs() < 1e-6);
    assert!((peak(&ghost.samples) - 0.2).abs() < 1e-6);
}

#[test]
fn a_too_hot_sample_comes_down() {
    let mut sound = tone(1.5);
    level([&mut sound]);
    assert!((peak(&sound.samples) - 1.0).abs() < 1e-6);
}

#[test]
fn silence_is_left_alone() {
    let mut silence = Audio {
        rate: 44_100,
        samples: vec![0.0; 100],
    };
    level([&mut silence]);
    assert!(silence.samples.iter().all(|&s| s == 0.0));
}
