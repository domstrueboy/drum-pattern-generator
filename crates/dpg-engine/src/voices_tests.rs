use super::*;

const RATE: u32 = 48_000;
const FADE: usize = 240; // 5 ms at 48 kHz
const FULL: [f32; CHANNELS] = [1.0; CHANNELS];

/// Sample n is a constant n + 1 over `len` frames, so a mix shows who
/// played: sample 0 is all 1.0, sample 1 all 2.0...
fn samples(len: usize) -> Vec<Vec<f32>> {
    (0..64).map(|n| vec![(n + 1) as f32; len]).collect()
}

fn trigger(sample: u32, channel: Channel, gain: f32, offset: usize) -> Trigger {
    Trigger {
        sample: SampleId(sample),
        channel,
        gain,
        offset,
    }
}

/// Mixes one mono block of `frames` at fixed gains.
fn mix(voices: &mut Voices, data: &[Vec<f32>], frames: usize, gains: [f32; CHANNELS]) -> Vec<f32> {
    let mut out = vec![0.0; frames];
    voices.mix(&mut out, 1, &gains, &gains, |id| &data[id.0 as usize]);
    out
}

#[test]
fn a_voice_starts_at_its_offset_and_carries_into_the_next_block() {
    let data = samples(6);
    let mut voices = Voices::new(RATE);
    voices.trigger(trigger(0, Channel::Snare, 0.5, 2));
    assert_eq!(mix(&mut voices, &data, 4, FULL), [0.0, 0.0, 0.5, 0.5]);
    assert_eq!(mix(&mut voices, &data, 4, FULL), [0.5, 0.5, 0.5, 0.5]);
    assert_eq!(mix(&mut voices, &data, 4, FULL), [0.0; 4]);
    assert_eq!(voices.active(), 0);
}

#[test]
fn an_offset_past_the_block_waits_for_a_later_one() {
    let data = samples(2);
    let mut voices = Voices::new(RATE);
    voices.trigger(trigger(0, Channel::Kick, 1.0, 5));
    assert_eq!(mix(&mut voices, &data, 4, FULL), [0.0; 4]);
    assert_eq!(mix(&mut voices, &data, 4, FULL), [0.0, 1.0, 1.0, 0.0]);
}

#[test]
fn voices_add_up_each_with_its_channel_gain() {
    let data = samples(4);
    let mut voices = Voices::new(RATE);
    voices.trigger(trigger(0, Channel::Kick, 1.0, 0)); // 1.0
    voices.trigger(trigger(1, Channel::Snare, 0.5, 0)); // 2.0 * 0.5
    let mut gains = FULL;
    gains[Channel::Snare.index()] = 0.25;
    assert_eq!(mix(&mut voices, &data, 2, gains), [1.25, 1.25]);
}

#[test]
fn a_gain_change_slides_across_the_block() {
    let data = samples(8);
    let mut voices = Voices::new(RATE);
    voices.trigger(trigger(0, Channel::HiHat, 1.0, 0));
    let mut out = vec![0.0; 4];
    let mut to = FULL;
    to[Channel::HiHat.index()] = 0.0;
    voices.mix(&mut out, 1, &FULL, &to, |id| &data[id.0 as usize]);
    assert_eq!(out, [1.0, 0.75, 0.5, 0.25]);
}

#[test]
fn every_output_channel_gets_the_same() {
    let data = samples(2);
    let mut voices = Voices::new(RATE);
    voices.trigger(trigger(2, Channel::Crash, 1.0, 1));
    let mut out = vec![0.0; 6];
    voices.mix(&mut out, 2, &FULL, &FULL, |id| &data[id.0 as usize]);
    assert_eq!(out, [0.0, 0.0, 3.0, 3.0, 3.0, 3.0]);
}

#[test]
fn a_full_pool_fades_the_oldest_to_make_room() {
    let data = samples(10_000);
    let mut voices = Voices::new(RATE);
    for n in 0..VOICES as u32 {
        voices.trigger(trigger(n, Channel::Snare, 1.0, 0));
    }
    voices.trigger(trigger(VOICES as u32, Channel::Snare, 1.0, 0));
    assert_eq!(voices.active(), VOICES + 1);
    // The oldest (sample 0) fades within 5 ms; then it's gone.
    mix(&mut voices, &data, FADE + 10, FULL);
    assert_eq!(voices.active(), VOICES);
    assert!(!voices.any(|id| id == SampleId(0)));
    assert!(voices.any(|id| id == SampleId(VOICES as u32)));
}

#[test]
fn a_fade_goes_down_smoothly_to_silence() {
    let data = samples(10_000);
    let mut voices = Voices::new(RATE);
    voices.trigger(trigger(0, Channel::Kick, 1.0, 0));
    voices.release_all();
    let out = mix(&mut voices, &data, FADE + 10, FULL);
    assert_eq!(out[0], 1.0);
    assert!(out[..FADE].windows(2).all(|w| w[1] < w[0]));
    assert!(out[FADE..].iter().all(|&s| s == 0.0));
    assert_eq!(voices.active(), 0);
}

#[test]
fn with_every_fade_slot_taken_the_oldest_fade_is_cut() {
    let mut voices = Voices::new(RATE);
    for n in 0..(VOICES + FADING + 3) as u32 {
        voices.trigger(trigger(n % 60, Channel::HiHat, 1.0, 0));
    }
    assert_eq!(voices.active(), VOICES + FADING);
    // The newest is always among them.
    assert!(voices.any(|id| id == SampleId((VOICES + FADING + 2) as u32)));
}

#[test]
fn stop_fades_everything_out() {
    let data = samples(10_000);
    let mut voices = Voices::new(RATE);
    for n in 0..5 {
        voices.trigger(trigger(n, Channel::Snare, 1.0, 0));
    }
    voices.release_all();
    mix(&mut voices, &data, FADE, FULL);
    assert_eq!(voices.active(), 0);
}

#[test]
fn a_hit_due_later_in_the_block_never_sounds_after_stop() {
    let data = samples(100);
    let mut voices = Voices::new(RATE);
    voices.trigger(trigger(0, Channel::Snare, 1.0, 1000));
    voices.release_all();
    assert_eq!(voices.active(), 0);
    assert!(
        mix(&mut voices, &data, 2000, FULL)
            .iter()
            .all(|&s| s == 0.0)
    );
}
