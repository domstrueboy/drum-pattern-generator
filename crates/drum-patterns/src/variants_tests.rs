use std::collections::HashSet;

use rand::RngExt;
use rand::rngs::StdRng;

use super::*;

fn first_draw(v: &Variants, index: u8) -> u64 {
    v.rng::<StdRng>(index).random()
}

#[test]
fn holds_one_to_twenty_four() {
    assert_eq!(Variants::new(1, 0).count(), 1);
    assert_eq!(Variants::new(1, 12).count(), 12);
    assert_eq!(Variants::new(1, 99).count(), 24);
}

#[test]
fn each_press_steps_to_the_next_then_wraps() {
    let v = Variants::new(1, 3);
    assert_eq!([v.next(0), v.next(1), v.next(2)], [1, 2, 0]);
}

#[test]
fn a_set_of_one_stays_put() {
    assert_eq!(Variants::new(1, 1).next(0), 0);
}

#[test]
fn the_same_variant_draws_the_same() {
    let v = Variants::new(42, 8);
    assert_eq!(first_draw(&v, 3), first_draw(&v, 3));
    assert_eq!(first_draw(&v, 3), first_draw(&Variants::new(42, 8), 3));
}

#[test]
fn variants_of_a_set_differ() {
    let v = Variants::new(42, 24);
    let draws: HashSet<u64> = (0..24).map(|i| first_draw(&v, i)).collect();
    assert_eq!(draws.len(), 24);
}

#[test]
fn sets_with_other_seeds_differ() {
    assert_ne!(
        first_draw(&Variants::new(1, 4), 0),
        first_draw(&Variants::new(2, 4), 0)
    );
}

#[test]
fn an_index_past_the_count_wraps() {
    let v = Variants::new(7, 5);
    assert_eq!(first_draw(&v, 6), first_draw(&v, 1));
}
