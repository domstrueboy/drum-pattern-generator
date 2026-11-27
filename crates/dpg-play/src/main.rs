//! Listen on a desktop: generates grooves, shows them as text and plays
//! them through a local kit on the default output. A dev tool for judging
//! levels and rules by ear - the kit stays on this machine.
//!
//! `dpg-play <kit folder>`, or set `DPG_KIT_DIR`.

mod command;
mod grid;
mod kit;

use std::io::BufRead;
use std::process::ExitCode;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use dpg_engine::{Engine, Remote};
use dpg_hal::CpalOutput;
use drum_patterns::Groove;

use command::{Command, Fader, HELP};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let dir = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("DPG_KIT_DIR").ok())
        .ok_or("usage: dpg-play <kit folder> (or set DPG_KIT_DIR)")?;

    let mut remote = None;
    let output = CpalOutput::open(|info| {
        let (engine, r) = Engine::new(info.rate);
        remote = Some(r);
        engine
    })
    .map_err(|e| e.to_string())?;
    let mut remote = remote.expect("open built the engine");
    let info = output.info();
    println!(
        "{} - {} Hz, {} channels",
        info.device, info.rate, info.channels
    );
    remote
        .set_kit(kit::load(&dir, info.rate)?)
        .map_err(|e| e.to_string())?;

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let mut groove = Groove {
        seed,
        ..Groove::default()
    };
    println!("{HELP}\n");
    send(&mut remote, &groove)?;

    let started = Instant::now();
    for line in std::io::stdin().lock().lines() {
        let line = line.map_err(|e| e.to_string())?;
        remote.poll();
        let command = match Command::parse(&line) {
            Ok(command) => command,
            Err(e) => {
                println!("{e}");
                continue;
            }
        };
        let regroove = apply(command, &mut groove, &mut remote, started);
        if command == Command::Quit {
            break;
        }
        if regroove {
            send(&mut remote, &groove)?;
        } else {
            status(&remote, &output);
        }
    }
    Ok(())
}

/// Carries out `command`. Returns whether the groove changed.
fn apply(command: Command, groove: &mut Groove, remote: &mut Remote, started: Instant) -> bool {
    use Command::*;
    match command {
        NewGroove => groove.seed = groove.seed.wrapping_add(1),
        Feel(feel) => groove.feel = feel,
        Accents(n) => groove.accents = n,
        Ghosts(n, level) => {
            groove.ghosts = n;
            groove.ghost_level = level.unwrap_or(groove.ghost_level);
        }
        Kicks(base, extra, level) => {
            groove.base_kicks = base;
            groove.extra_kicks = extra.unwrap_or(groove.extra_kicks);
            groove.extra_kick_level = level.unwrap_or(groove.extra_kick_level);
        }
        HiHat(pulse) => groove.hi_hat = pulse,
        Random(density) => groove.hi_hat_random = density,
        Bars(n) => groove.bars = n,
        StartStop if remote.is_playing() => {
            remote.stop();
            return false;
        }
        StartStop => {
            remote.play();
            return false;
        }
        Bpm(bpm) => {
            remote.set_bpm(bpm);
            return false;
        }
        Tap => {
            remote.tap(started.elapsed().as_secs_f64());
            return false;
        }
        Metronome => {
            remote.set_metronome(!remote.metronome());
            return false;
        }
        Subdivision(s) => {
            remote.set_subdivision(s);
            return false;
        }
        BarSkip(skip) => {
            remote.set_bar_skip(skip);
            return false;
        }
        Crash(rule) => {
            remote.set_crash(rule);
            return false;
        }
        Level(Fader::Drums, position) => {
            remote.mix().set_drums(position);
            return false;
        }
        Level(Fader::Channel(channel), position) => {
            remote.mix().set_level(channel, position);
            return false;
        }
        Solo(channel) => {
            let on = remote.mix().snapshot().solo & 1 << channel.index() == 0;
            remote.mix().set_solo(channel, on);
            return false;
        }
        Mute(channel) => {
            let on = remote.mix().snapshot().mute & 1 << channel.index() == 0;
            remote.mix().set_mute(channel, on);
            return false;
        }
        Help => {
            println!("{HELP}");
            return false;
        }
        Quit => return false,
    }
    true
}

/// Builds the groove, prints it and hands it to the engine.
fn send(remote: &mut Remote, groove: &Groove) -> Result<(), String> {
    let laps = groove.build_laps().map_err(|e| e.to_string())?;
    println!("\nseed {}\n{}\n", groove.seed, grid::grid(&laps));
    remote.set_laps(laps);
    Ok(())
}

fn status(remote: &Remote, output: &CpalOutput) {
    let mix = remote.mix().snapshot();
    let metronome = if remote.metronome() {
        format!("{} a beat", remote.subdivision().clicks())
    } else {
        "off".into()
    };
    let skip = remote
        .bar_skip()
        .map_or("off".into(), |s| format!("{}-{}", s.play(), s.skip()));
    let stats = output.stats();
    println!(
        "{} | {:.0} BPM | metronome {metronome} | bar skip {skip} | crash {:?} | \
         faders {:?} drums {} | solo {:#07b} mute {:#07b} | {:.0} ms, {} xruns",
        if remote.is_playing() {
            "playing"
        } else {
            "stopped"
        },
        remote.bpm(),
        remote.crash(),
        mix.levels,
        mix.drums,
        mix.solo,
        mix.mute,
        stats.latency() * 1000.0,
        stats.xruns(),
    );
}
