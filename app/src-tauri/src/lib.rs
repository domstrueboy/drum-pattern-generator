//! The app's shell: opens the output, starts a session on it and passes
//! the UI's commands to it. Every command returns a fresh
//! snapshot; the UI only renders it. What a command does is
//! `dpg-session`'s.
//!
//! An output that is lost - a phone call, headphones pulled - is opened
//! again on the next command or refresh, and the session carries on in it.

use std::sync::Mutex;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use dpg_engine::{Engine, Remote};
use dpg_hal::CpalOutput;
use dpg_session::{Command, Playhead, Session, Snapshot};
use serde::Serialize;
use tauri::State;

/// The open output and the session driving it.
struct Audio {
    output: CpalOutput,
    session: Session,
}

/// The audio, once an output has opened, and the last reason one
/// couldn't. The mutex is for UI-side calls only; the audio thread never
/// sees it.
struct Shell {
    audio: Option<Audio>,
    error: Option<String>,
}

/// The shell and the clock Tap tempo reads.
struct App {
    shell: Mutex<Shell>,
    started: Instant,
}

/// The session's state plus the output's.
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct AppSnapshot {
    /// Why there's no sound, if there isn't.
    error: Option<String>,
    device: String,
    rate: u32,
    latency_ms: f64,
    xruns: u32,
    #[serde(flatten)]
    session: Option<Snapshot>,
}

/// Opens the default output with a new engine; the session gives it a kit.
fn open_output() -> Result<(CpalOutput, Remote), String> {
    let mut remote = None;
    let output = CpalOutput::open(|info| {
        let (engine, r) = Engine::new(info.rate);
        remote = Some(r);
        engine
    })
    .map_err(|e| e.to_string())?;
    // The session makes the kit and sends it.
    Ok((output, remote.expect("open built the engine")))
}

impl Shell {
    /// Opens an output if there's none or it was lost. The old one stays
    /// until a new one opens, so a failure is tried again next time.
    fn connect(&mut self) {
        let lost = self.audio.as_ref().is_some_and(|a| a.output.stats().lost());
        if self.audio.is_some() && !lost {
            return;
        }
        match open_output() {
            Ok((output, remote)) => {
                self.error = None;
                match &mut self.audio {
                    Some(audio) => {
                        audio.session.reconnect(remote);
                        audio.output = output;
                    }
                    None => {
                        let seed = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map_or(0, |d| d.as_secs());
                        self.audio = Some(Audio {
                            output,
                            session: Session::new(remote, seed),
                        });
                    }
                }
            }
            Err(e) => self.error = Some(e),
        }
    }
}

/// Runs `f` on the session, once the output is open, then reports the
/// state.
fn with_session(app: State<App>, f: impl FnOnce(&mut Session, f64)) -> AppSnapshot {
    let now = app.started.elapsed().as_secs_f64();
    let mut shell = app.shell.lock().expect("no command panics holding it");
    shell.connect();
    let error = shell.error.clone();
    let Some(audio) = &mut shell.audio else {
        return AppSnapshot {
            error,
            ..AppSnapshot::default()
        };
    };
    f(&mut audio.session, now);
    let info = audio.output.info();
    let stats = audio.output.stats();
    AppSnapshot {
        error: error.or_else(|| {
            stats
                .lost()
                .then(|| "The audio output was lost".to_string())
        }),
        device: info.device.clone(),
        rate: info.rate,
        latency_ms: stats.latency() * 1000.0,
        xruns: stats.xruns(),
        session: Some(audio.session.snapshot()),
    }
}

#[tauri::command]
fn snapshot(app: State<App>) -> AppSnapshot {
    with_session(app, |session, _| {
        session.remote().poll();
    })
}

/// Where the notation's line is: asked once a screen frame while playing,
/// so it skips the snapshot and reopening a lost output.
#[tauri::command]
fn playhead(app: State<App>) -> Option<Playhead> {
    let now = app.started.elapsed().as_secs_f64();
    let mut shell = app.shell.lock().expect("no command panics holding it");
    shell.audio.as_mut()?.session.playhead(now)
}

#[tauri::command]
fn send(app: State<App>, command: Command) -> AppSnapshot {
    with_session(app, |session, now| session.apply(command, now))
}

/// Starts the app: the audio first, then the window.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut shell = Shell {
        audio: None,
        error: None,
    };
    shell.connect();
    tauri::Builder::default()
        .manage(App {
            shell: Mutex::new(shell),
            started: Instant::now(),
        })
        .invoke_handler(tauri::generate_handler![snapshot, send, playhead])
        .run(tauri::generate_context!())
        .expect("the window opens");
}
