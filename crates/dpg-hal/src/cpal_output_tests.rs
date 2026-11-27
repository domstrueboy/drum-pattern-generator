use std::sync::atomic::AtomicUsize;
use std::time::Duration;

use super::*;
use crate::Block;

/// Counts the frames it is asked for, and plays silence.
struct Counter(Arc<AtomicUsize>);

impl Render for Counter {
    fn render(&mut self, block: Block<'_>) {
        self.0
            .fetch_add(block.out.len() / block.channels, Ordering::Relaxed);
    }
}

/// Opens the real default output and plays silence for half a second.
/// Needs a sound device, so it is run by hand:
/// `cargo test -p dpg-hal real_output -- --ignored --nocapture`.
#[test]
#[ignore = "needs an audio device"]
fn real_output_asks_for_frames() {
    let frames = Arc::new(AtomicUsize::new(0));
    let output = CpalOutput::open(|_| Counter(frames.clone())).unwrap();
    std::thread::sleep(Duration::from_millis(500));
    let asked = frames.load(Ordering::Relaxed);
    let info = output.info().clone();
    let stats = output.stats();
    println!(
        "{} at {} Hz, {} ch: {asked} frames in 0.5 s, latency {:.1} ms, {} xruns",
        info.device,
        info.rate,
        info.channels,
        stats.latency() * 1000.0,
        stats.xruns()
    );
    drop(output);
    // Half a second's worth, give or take the device's buffering.
    let expected = info.rate as usize / 2;
    assert!(
        asked > expected / 2 && asked < expected * 2,
        "{asked} frames"
    );

    // Dropped: the stream has stopped asking.
    let after = frames.load(Ordering::Relaxed);
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(frames.load(Ordering::Relaxed), after);
}
