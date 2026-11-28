use super::*;

const RATE: u32 = 48_000;

fn peak(samples: &[f32]) -> f32 {
    samples.iter().fold(0f32, |m, x| m.max(x.abs()))
}

fn crossings(samples: &[f32]) -> usize {
    samples
        .windows(2)
        .filter(|w| (w[0] < 0.0) != (w[1] < 0.0))
        .count()
}

#[test]
fn every_sound_peaks_at_full_scale_and_dies_away() {
    let clicks = Clicks::new(RATE);
    for sound in ClickSound::ALL {
        for click in [Click::Bar, Click::Beat] {
            let samples = clicks.sound(sound, click).0;
            assert!((peak(samples) - 1.0).abs() < 1e-4, "{sound:?} {click:?}");
            let last_ms = &samples[samples.len() - 48..];
            assert!(peak(last_ms) < 0.02, "{sound:?} {click:?} rings to the end");
        }
    }
}

#[test]
fn every_bar_click_is_higher() {
    let clicks = Clicks::new(RATE);
    for sound in ClickSound::ALL {
        let high = clicks.sound(sound, Click::Bar).0;
        let low = clicks.sound(sound, Click::Beat).0;
        assert_eq!(high.len(), low.len(), "{sound:?}");
        assert!(
            crossings(high) * 10 > crossings(low) * 11,
            "{sound:?}: {} vs {}",
            crossings(high),
            crossings(low)
        );
    }
}

#[test]
fn subdivisions_play_quieter() {
    let clicks = Clicks::new(RATE);
    for sound in ClickSound::ALL {
        let (_, beat) = clicks.sound(sound, Click::Beat);
        let (_, between) = clicks.sound(sound, Click::Between);
        assert!((20.0 * (between / beat).log10() - SUBDIVISION_DB).abs() < 1e-4);
    }
}

#[test]
fn the_sounds_differ() {
    let clicks = Clicks::new(RATE);
    for (i, a) in ClickSound::ALL.iter().enumerate() {
        for b in &ClickSound::ALL[i + 1..] {
            assert_ne!(
                clicks.sound(*a, Click::Beat).0,
                clicks.sound(*b, Click::Beat).0,
                "{a:?} and {b:?}"
            );
        }
    }
}

#[test]
fn the_button_steps_through_all_six_and_round() {
    let mut sound = ClickSound::default();
    let mut seen = vec![sound];
    for _ in 1..ClickSound::ALL.len() {
        sound = sound.next();
        seen.push(sound);
    }
    assert_eq!(seen, ClickSound::ALL);
    assert_eq!(sound.next(), ClickSound::Click);
}

#[test]
fn sounds_survive_an_atomic() {
    for sound in ClickSound::ALL {
        assert_eq!(ClickSound::from_index(sound.index()), sound);
    }
}

#[test]
fn the_electronic_beep_is_bright_and_short() {
    let clicks = Clicks::new(RATE);
    let beep = clicks.sound(ClickSound::Electronic, Click::Beat).0;
    let click = clicks.sound(ClickSound::Click, Click::Beat).0;
    assert!(beep.len() < click.len() * 2);
    // It holds at full level where the plain click has died away.
    let at_20ms = |s: &[f32]| peak(&s[960..1008]);
    assert!(at_20ms(beep) > 0.5);
    assert!(at_20ms(click) < 0.1);
}
