use super::*;
use crate::Block;

/// Writes each frame's index into every channel, and remembers what it
/// was told.
#[derive(Default)]
struct Ramp {
    next: usize,
    blocks: Vec<(usize, f64)>,
}

impl Render for Ramp {
    fn render(&mut self, block: Block<'_>) {
        let frames = block.out.len() / block.channels;
        self.blocks.push((frames, block.latency));
        for frame in block.out.chunks_mut(block.channels) {
            frame.fill(self.next as f32 / 1e6);
            self.next += 1;
        }
    }
}

#[test]
fn pulls_interleaved_frames_in_order() {
    let mut output = MockOutput::new(48_000, 2, Ramp::default());
    let out = output.pull(3);
    assert_eq!(out, [0.0, 0.0, 1e-6, 1e-6, 2e-6, 2e-6]);
    assert_eq!(output.pull(1), [3e-6, 3e-6]);
}

#[test]
fn a_long_pull_comes_in_blocks_that_carry_on() {
    let mut output = MockOutput::new(48_000, 1, Ramp::default());
    let out = output.pull(MAX_BLOCK_FRAMES * 2 + 10);
    let frames: Vec<usize> = output.render().blocks.iter().map(|b| b.0).collect();
    assert_eq!(frames, [MAX_BLOCK_FRAMES, MAX_BLOCK_FRAMES, 10]);
    assert_eq!(out[MAX_BLOCK_FRAMES], MAX_BLOCK_FRAMES as f32 / 1e6);
}

#[test]
fn later_blocks_are_heard_later() {
    let mut output = MockOutput::new(48_000, 1, Ramp::default());
    output.set_latency(0.01);
    output.pull(MAX_BLOCK_FRAMES + 1);
    let latencies: Vec<f64> = output.render().blocks.iter().map(|b| b.1).collect();
    let block_seconds = MAX_BLOCK_FRAMES as f64 / 48_000.0;
    assert_eq!(latencies[0], 0.01);
    assert!((latencies[1] - (0.01 + block_seconds)).abs() < 1e-9);
    assert!((output.stats().latency() - 0.01).abs() < 1e-6);
}

/// Plays a fixed value.
struct Loud(f32);

impl Render for Loud {
    fn render(&mut self, block: Block<'_>) {
        block.out.fill(self.0);
    }
}

#[test]
fn the_output_is_clipped_to_full_scale() {
    assert_eq!(MockOutput::new(48_000, 1, Loud(1.7)).pull(2), [1.0, 1.0]);
    assert_eq!(MockOutput::new(48_000, 1, Loud(-3.0)).pull(1), [-1.0]);
    assert_eq!(MockOutput::new(48_000, 1, Loud(0.5)).pull(1), [0.5]);
}

#[test]
fn reports_what_it_pretends_to_be() {
    let output = MockOutput::new(44_100, 2, Loud(0.0));
    assert_eq!(output.info().rate, 44_100);
    assert_eq!(output.info().channels, 2);
    assert_eq!(output.stats().xruns(), 0);
    assert!(!output.stats().lost());
}
