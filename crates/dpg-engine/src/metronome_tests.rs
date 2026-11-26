use super::*;

const FOUR_FOUR: TimeSignature = TimeSignature::FOUR_FOUR;

/// The ticks in a bar of 4/4 that click, with their kind.
fn bar(subdivision: Subdivision) -> Vec<(u64, Click)> {
    (0..48)
        .filter_map(|t| click_at(t, FOUR_FOUR, subdivision).map(|c| (t, c)))
        .collect()
}

#[test]
fn quarters_click_the_beats_the_first_higher() {
    use Click::*;
    assert_eq!(
        bar(Subdivision::Quarter),
        [(0, Bar), (12, Beat), (24, Beat), (36, Beat)]
    );
}

#[test]
fn each_subdivision_splits_the_beat_evenly() {
    for (subdivision, ticks) in [
        (Subdivision::Eighth, vec![0, 6]),
        (Subdivision::Triplet, vec![0, 4, 8]),
        (Subdivision::Sixteenth, vec![0, 3, 6, 9]),
    ] {
        let first_beat: Vec<_> = bar(subdivision)
            .into_iter()
            .take_while(|&(t, _)| t < 12)
            .map(|(t, _)| t)
            .collect();
        assert_eq!(first_beat, ticks, "{subdivision:?}");
        assert_eq!(bar(subdivision).len(), 4 * subdivision.clicks() as usize);
    }
}

#[test]
fn an_eighth_falls_between_beats() {
    let kinds: Vec<_> = bar(Subdivision::Eighth)
        .into_iter()
        .map(|(_, c)| c)
        .collect();
    assert_eq!(kinds[1], Click::Between);
    assert_eq!(kinds[2], Click::Beat);
}

#[test]
fn every_bar_starts_high() {
    assert_eq!(
        click_at(48, FOUR_FOUR, Subdivision::Quarter),
        Some(Click::Bar)
    );
    assert_eq!(
        click_at(5 * 48 + 12, FOUR_FOUR, Subdivision::Quarter),
        Some(Click::Beat)
    );
}

#[test]
fn in_six_eight_the_beat_is_an_eighth() {
    let six_eight = TimeSignature::new(6, 8).unwrap();
    let beats: Vec<_> = (0..36)
        .filter(|&t| click_at(t, six_eight, Subdivision::Quarter).is_some())
        .collect();
    assert_eq!(beats, [0, 6, 12, 18, 24, 30]);
    assert_eq!(
        click_at(2, six_eight, Subdivision::Triplet),
        Some(Click::Between)
    );
    // A 16th of an eighth is off the grid: beats only.
    let sixteenths = (0..36)
        .filter(|&t| click_at(t, six_eight, Subdivision::Sixteenth).is_some())
        .count();
    assert_eq!(sixteenths, 6);
}

#[test]
fn subdivisions_survive_an_atomic() {
    for subdivision in Subdivision::ALL {
        assert_eq!(Subdivision::from_index(subdivision.index()), subdivision);
    }
}

#[test]
fn the_bar_click_is_higher_and_subdivisions_quieter() {
    let clicks = Clicks::new(48_000);
    let (high, bar_gain) = clicks.sound(Click::Bar);
    let (low, beat_gain) = clicks.sound(Click::Beat);
    let (_, between_gain) = clicks.sound(Click::Between);
    assert_eq!(bar_gain, beat_gain);
    assert!((20.0 * (between_gain / beat_gain).log10() - SUBDIVISION_DB).abs() < 1e-4);
    assert!(crossings(high) > 3 * crossings(low) / 2, "an octave up");
}

#[test]
fn a_click_is_short_and_dies_away() {
    let rate = 48_000;
    let click = Clicks::new(rate).sound(Click::Beat).0.to_vec();
    assert_eq!(click.len(), (rate * CLICK_MS / 1000) as usize);
    let peak = |s: &[f32]| s.iter().fold(0f32, |m, x| m.max(x.abs()));
    assert!(peak(&click) > 0.5 && peak(&click) <= 1.0);
    assert!(
        peak(&click[click.len() - 48..]) < 0.002,
        "below -54 dB at the end"
    );
}

fn crossings(samples: &[f32]) -> usize {
    samples
        .windows(2)
        .filter(|w| (w[0] < 0.0) != (w[1] < 0.0))
        .count()
}
