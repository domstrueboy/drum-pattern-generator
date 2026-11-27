//! The app's shell: opens the output, plays a groove on the placeholder
//! kit and gives the UI its commands. Every command returns a fresh
//! snapshot of the state; the UI only renders it. The logic moves to `dpg-session`
//! in step 5b.

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use dpg_engine::{Engine, Remote};
use dpg_hal::CpalOutput;
use drum_patterns::Groove;
use serde::Serialize;
use tauri::State;

/// The open output and the engine's remote. The mutex is for UI-side
/// calls only; the audio thread never sees it.
struct Audio {
    output: CpalOutput,
    remote: Remote,
}

/// The audio, or why it couldn't start.
struct App(Mutex<Result<Audio, String>>);

/// What the screen shows.
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    /// Why there's no sound, if there isn't.
    error: Option<String>,
    playing: bool,
    bpm: f64,
    device: String,
    rate: u32,
    latency_ms: f64,
    xruns: u32,
}

/// Opens the default output, then loads the kit and a first groove.
fn open() -> Result<Audio, String> {
    let mut remote = None;
    let output = CpalOutput::open(|info| {
        let (engine, r) = Engine::new(info.rate);
        remote = Some(r);
        engine
    })
    .map_err(|e| e.to_string())?;
    let mut remote = remote.expect("open built the engine");
    remote
        .set_kit(dpg_samples::placeholder_kit(output.info().rate))
        .map_err(|e| e.to_string())?;
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let groove = Groove {
        seed,
        ..Groove::default()
    };
    remote.set_laps(groove.build_laps().map_err(|e| e.to_string())?);
    Ok(Audio { output, remote })
}

/// Runs `f` on the audio, if it opened, then reports the state.
fn with_audio(app: State<App>, f: impl FnOnce(&mut Audio)) -> Snapshot {
    let mut guard = app.0.lock().expect("no command panics holding it");
    let audio = match &mut *guard {
        Ok(audio) => audio,
        Err(e) => {
            return Snapshot {
                error: Some(e.clone()),
                ..Snapshot::default()
            };
        }
    };
    audio.remote.poll();
    f(audio);
    let info = audio.output.info();
    let stats = audio.output.stats();
    Snapshot {
        error: stats
            .lost()
            .then(|| "The audio output was lost".to_string()),
        playing: audio.remote.is_playing(),
        bpm: audio.remote.bpm(),
        device: info.device.clone(),
        rate: info.rate,
        latency_ms: stats.latency() * 1000.0,
        xruns: stats.xruns(),
    }
}

#[tauri::command]
fn snapshot(app: State<App>) -> Snapshot {
    with_audio(app, |_| {})
}

#[tauri::command]
fn start_stop(app: State<App>) -> Snapshot {
    with_audio(app, |audio| {
        if audio.remote.is_playing() {
            audio.remote.stop();
        } else {
            audio.remote.play();
        }
    })
}

#[tauri::command]
fn set_bpm(app: State<App>, bpm: f64) -> Snapshot {
    with_audio(app, |audio| audio.remote.set_bpm(bpm))
}

/// Starts the app: the audio first, then the window.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(App(Mutex::new(open())))
        .invoke_handler(tauri::generate_handler![snapshot, start_stop, set_bpm])
        .run(tauri::generate_context!())
        .expect("the window opens");
}
