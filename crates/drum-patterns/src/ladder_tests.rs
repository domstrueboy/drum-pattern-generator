use rand::SeedableRng;
use rand::rngs::StdRng;

use super::*;

#[test]
fn always_takes_one_newly_opened_position() {
    let easier = [3, 9, 15];
    let allowed = [3, 9, 15, 21];
    for seed in 0..50 {
        let chosen = pick(&easier, &allowed, 2, &mut StdRng::seed_from_u64(seed));
        assert!(chosen.contains(&21), "seed {seed}: {chosen:?}");
        assert_eq!(chosen.len(), 2);
    }
}

#[test]
fn picks_only_allowed_ticks_without_repeats() {
    let allowed = [0, 3, 6, 9, 12];
    for seed in 0..50 {
        let chosen = pick(&[], &allowed, 4, &mut StdRng::seed_from_u64(seed));
        assert!(chosen.iter().all(|t| allowed.contains(t)));
        assert!(chosen.windows(2).all(|w| w[0] < w[1]), "{chosen:?}");
    }
}

#[test]
fn gives_what_there_is_when_asked_for_more() {
    let chosen = pick(&[3], &[3, 9], 5, &mut StdRng::seed_from_u64(1));
    assert_eq!(chosen, [3, 9]);
}

#[test]
fn nothing_new_still_fills_from_the_allowed() {
    let chosen = pick(&[3, 9], &[3, 9], 1, &mut StdRng::seed_from_u64(1));
    assert_eq!(chosen.len(), 1);
}

#[test]
fn zero_asked_picks_nothing() {
    assert!(pick(&[], &[3, 9], 0, &mut StdRng::seed_from_u64(1)).is_empty());
}
