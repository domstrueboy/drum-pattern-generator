//! The voice pool: every sound playing right now, mixed into the block
//! with its channel's gain. Fixed size, so the audio thread never
//! allocates; when it is full, the oldest voice fades out to make room.

use crate::mix::{CHANNELS, Channel};

#[cfg(test)]
#[path = "voices_tests.rs"]
mod tests;

/// How many sounds play at once. A 16th hi-hat, snare and kick at 300 BPM
/// with long crash and cymbal tails stays well inside it.
pub const VOICES: usize = 32;
/// Extra slots for voices fading out, so a stolen voice can fade while
/// the new one starts.
const FADING: usize = 8;
/// How long a stolen or stopped voice takes to fade - short enough to
/// feel instant, long enough not to click.
pub const FADE_MS: u32 = 5;

/// Which sample a voice plays. The engine gives the numbers and looks them
/// up when mixing, so a voice never owns a sample - it can't be the one to
/// free it on the audio thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SampleId(pub u32);

/// A sound to start in the block being mixed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Trigger {
    /// What it plays.
    pub sample: SampleId,
    /// Where it is heard.
    pub channel: Channel,
    /// Its own gain, before the channel's: accent, normal or ghost.
    pub gain: f32,
    /// Frames from the start of the block.
    pub offset: usize,
}

#[derive(Debug, Clone, Copy)]
struct Voice {
    sample: SampleId,
    channel: Channel,
    gain: f32,
    /// The next sample frame to play.
    pos: usize,
    /// Frames into the current block before it starts; 0 once started.
    wait: usize,
    /// Frames of fade left, when fading.
    fade: Option<usize>,
    /// Started before any voice with a larger age.
    age: u64,
}

/// Every sound playing.
#[derive(Debug)]
pub struct Voices {
    slots: [Option<Voice>; VOICES + FADING],
    next_age: u64,
    fade_frames: usize,
}

impl Voices {
    /// An empty pool for a device at `rate`.
    pub fn new(rate: u32) -> Self {
        Self {
            slots: [None; VOICES + FADING],
            next_age: 0,
            fade_frames: (rate * FADE_MS / 1000).max(1) as usize,
        }
    }

    /// Starts a sound. If [`VOICES`] are already playing, the oldest fades
    /// out to make room.
    pub fn trigger(&mut self, trigger: Trigger) {
        let playing = self
            .slots
            .iter()
            .flatten()
            .filter(|v| v.fade.is_none())
            .count();
        if playing >= VOICES
            && let Some(oldest) = self.oldest(|v| v.fade.is_none())
        {
            self.slots[oldest].as_mut().expect("found it").fade = Some(self.fade_frames);
        }
        let slot = match self.slots.iter().position(Option::is_none) {
            Some(free) => free,
            // Every fade slot is taken too: cut the oldest fade short.
            None => self.oldest(|v| v.fade.is_some()).expect("the pool is full"),
        };
        self.slots[slot] = Some(Voice {
            sample: trigger.sample,
            channel: trigger.channel,
            gain: trigger.gain,
            pos: 0,
            wait: trigger.offset,
            fade: None,
            age: self.next_age,
        });
        self.next_age += 1;
    }

    /// Stop: every voice fades out, and one not started yet never does.
    pub fn release_all(&mut self) {
        let frames = self.fade_frames;
        for slot in &mut self.slots {
            match slot {
                Some(voice) if voice.wait > 0 => *slot = None,
                Some(voice) => voice.fade = Some(voice.fade.map_or(frames, |f| f.min(frames))),
                None => {}
            }
        }
    }

    /// Adds every voice into `out` (interleaved, `channels` a frame, the
    /// same on each), with each channel's gain moving from `from` at the
    /// block's start to `to` at its end. `sample` looks a voice's sample
    /// up. Finished voices are dropped. Allocates nothing.
    pub fn mix<'s>(
        &mut self,
        out: &mut [f32],
        channels: usize,
        from: &[f32; CHANNELS],
        to: &[f32; CHANNELS],
        sample: impl Fn(SampleId) -> &'s [f32],
    ) {
        let frames = out.len() / channels;
        let fade_frames = self.fade_frames as f32;
        for slot in &mut self.slots {
            let Some(voice) = slot else { continue };
            let data = sample(voice.sample);
            let (a, b) = (from[voice.channel.index()], to[voice.channel.index()]);
            let start = voice.wait.min(frames);
            for (i, frame) in out.chunks_exact_mut(channels).enumerate().skip(start) {
                let Some(&s) = data.get(voice.pos) else { break };
                let mut gain = voice.gain * (a + (b - a) * i as f32 / frames as f32);
                if let Some(left) = voice.fade.as_mut() {
                    if *left == 0 {
                        break;
                    }
                    gain *= *left as f32 / fade_frames;
                    *left -= 1;
                }
                frame.iter_mut().for_each(|o| *o += s * gain);
                voice.pos += 1;
            }
            voice.wait = voice.wait.saturating_sub(frames);
            if voice.pos >= data.len() || voice.fade == Some(0) {
                *slot = None;
            }
        }
    }

    /// How many voices are sounding, fading ones included.
    pub fn active(&self) -> usize {
        self.slots.iter().flatten().count()
    }

    /// Whether any voice plays a sample `matches` picks - so the engine
    /// knows when a replaced kit has fallen silent.
    pub fn any(&self, matches: impl Fn(SampleId) -> bool) -> bool {
        self.slots.iter().flatten().any(|v| matches(v.sample))
    }

    fn oldest(&self, pick: impl Fn(&Voice) -> bool) -> Option<usize> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(i, v)| v.filter(|v| pick(v)).map(|v| (i, v.age)))
            .min_by_key(|&(_, age)| age)
            .map(|(i, _)| i)
    }
}
