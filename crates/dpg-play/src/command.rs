//! What the player understands: one command a line, typed in the
//! terminal. An empty line asks for a new groove.

use dpg_engine::{BarSkip, Channel, CrashRule, Subdivision};
use drum_patterns::{Feel, NoteValue};

#[cfg(test)]
#[path = "command_tests.rs"]
mod tests;

/// Printed for `?`.
pub const HELP: &str = "\
Enter            new groove (next seed)
p                Start / Stop
b 92             tempo
t                tap tempo (t + Enter, again and again)
m | m 8          metronome on/off | subdivision 4, 8, 8t, 16
f basic|half|double            snare feel
a 2              snare accents a bar
g 3 [level]      ghost notes a bar, level 1-4
k 2 [extra] [level]            base kicks, extra kicks, level 1-3
h 8 | 16 | 4t | 8t | 16t | 4. | 8.   hi-hat pulse
r | r 30 | r off Random hi-hat, density in %
n 2              bars (1-4)
x 2-1 | x off    bar skip
c once | c 4     crash: once, or every N bars
v kick 0.5       fader 0-1: crash, hihat, snare, kick, drums, metronome
s kick | u kick  solo | mute on/off
? | q            help | quit";

/// One fader of the mixer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fader {
    /// A channel's own.
    Channel(Channel),
    /// The Drums fader, over every drum.
    Drums,
}

/// A typed command.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    /// A new groove with the same settings.
    NewGroove,
    /// Start, or Stop while playing.
    StartStop,
    /// A tempo.
    Bpm(f64),
    /// A tap of Tap tempo.
    Tap,
    /// The metronome on or off.
    Metronome,
    /// The metronome's clicks a beat.
    Subdivision(Subdivision),
    /// The snare's feel.
    Feel(Feel),
    /// Snare accents a bar.
    Accents(u8),
    /// Ghost notes a bar, and their level.
    Ghosts(u8, Option<u8>),
    /// Base kicks, extra kicks and the extra kicks' level.
    Kicks(u8, Option<u8>, Option<u8>),
    /// The hi-hat pulse.
    HiHat(NoteValue),
    /// Random on at a density, or off.
    Random(Option<u8>),
    /// The pattern's length.
    Bars(u8),
    /// Bar skip on with a round, or off.
    BarSkip(Option<BarSkip>),
    /// Where the crash plays.
    Crash(CrashRule),
    /// A fader to a position, 0 to 1.
    Level(Fader, f32),
    /// Solo a channel, or un-solo it.
    Solo(Channel),
    /// Mute a channel, or un-mute it.
    Mute(Channel),
    /// Show the commands.
    Help,
    /// Leave.
    Quit,
}

impl Command {
    /// Reads a line, or says what is wrong with it.
    pub fn parse(line: &str) -> Result<Self, String> {
        let words: Vec<&str> = line.split_whitespace().collect();
        let Some((&name, args)) = words.split_first() else {
            return Ok(Self::NewGroove);
        };
        let arg = |i: usize| args.get(i).copied();
        let need = |i: usize| arg(i).ok_or_else(|| format!("{name}: missing a value - ? for help"));
        let command = match name {
            "p" => Self::StartStop,
            "b" => Self::Bpm(number(need(0)?)?),
            "t" => Self::Tap,
            "m" => match arg(0) {
                None => Self::Metronome,
                Some(s) => Self::Subdivision(subdivision(s)?),
            },
            "f" => Self::Feel(feel(need(0)?)?),
            "a" => Self::Accents(number(need(0)?)?),
            "g" => Self::Ghosts(number(need(0)?)?, arg(1).map(number).transpose()?),
            "k" => Self::Kicks(
                number(need(0)?)?,
                arg(1).map(number).transpose()?,
                arg(2).map(number).transpose()?,
            ),
            "h" => Self::HiHat(pulse(need(0)?)?),
            "r" => match arg(0) {
                None => Self::Random(Some(drum_patterns::HI_HAT_DENSITY)),
                Some("off") => Self::Random(None),
                Some(s) => Self::Random(Some(number(s)?)),
            },
            "n" => Self::Bars(number(need(0)?)?),
            "x" => Self::BarSkip(bar_skip(need(0)?)?),
            "c" => Self::Crash(match need(0)? {
                "once" => CrashRule::Once,
                s => CrashRule::Every(number(s)?),
            }),
            "v" => Self::Level(fader(need(0)?)?, number(need(1)?)?),
            "s" => Self::Solo(channel(need(0)?)?),
            "u" => Self::Mute(channel(need(0)?)?),
            "?" => Self::Help,
            "q" => Self::Quit,
            _ => return Err(format!("{name}: no such command - ? for help")),
        };
        Ok(command)
    }
}

fn number<T: std::str::FromStr>(s: &str) -> Result<T, String> {
    s.parse()
        .map_err(|_| format!("{s}: not a number that fits here"))
}

fn subdivision(s: &str) -> Result<Subdivision, String> {
    Ok(match s {
        "4" => Subdivision::Quarter,
        "8" => Subdivision::Eighth,
        "8t" => Subdivision::Triplet,
        "16" => Subdivision::Sixteenth,
        _ => return Err(format!("{s}: the metronome takes 4, 8, 8t or 16")),
    })
}

fn feel(s: &str) -> Result<Feel, String> {
    Ok(match s {
        "basic" => Feel::Basic,
        "half" => Feel::HalfTime,
        "double" => Feel::DoubleTime,
        _ => return Err(format!("{s}: the feel is basic, half or double")),
    })
}

fn pulse(s: &str) -> Result<NoteValue, String> {
    Ok(match s {
        "4" => NoteValue::Quarter,
        "8" => NoteValue::Eighth,
        "16" => NoteValue::Sixteenth,
        "4t" => NoteValue::QuarterTriplet,
        "8t" => NoteValue::EighthTriplet,
        "16t" => NoteValue::SixteenthTriplet,
        "4." => NoteValue::DottedQuarter,
        "8." => NoteValue::DottedEighth,
        _ => return Err(format!("{s}: the pulse is 4, 8, 16, 4t, 8t, 16t, 4. or 8.")),
    })
}

fn bar_skip(s: &str) -> Result<Option<BarSkip>, String> {
    if s == "off" {
        return Ok(None);
    }
    let (play, skip) = s
        .split_once('-')
        .ok_or_else(|| format!("{s}: bar skip is like 2-1, or off"))?;
    BarSkip::new(number(play)?, number(skip)?)
        .map(Some)
        .ok_or_else(|| format!("{s}: both sides at least 1"))
}

fn channel(s: &str) -> Result<Channel, String> {
    Ok(match s {
        "crash" => Channel::Crash,
        "hihat" => Channel::HiHat,
        "snare" => Channel::Snare,
        "kick" => Channel::Kick,
        "metronome" => Channel::Metronome,
        _ => return Err(format!("{s}: crash, hihat, snare, kick or metronome")),
    })
}

fn fader(s: &str) -> Result<Fader, String> {
    if s == "drums" {
        Ok(Fader::Drums)
    } else {
        channel(s).map(Fader::Channel)
    }
}
