//! Step 1 spike: a Tauri shell over a cpal loop, to decide whether Tauri
//! is the shell for the app. Throwaway, never merged.

mod audio;

use std::sync::Mutex;
use std::sync::atomic::Ordering;

use serde::Serialize;
use tauri::State;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_fs::FsExt;

/// The engine, or why it couldn't start. The mutex guards only UI-side
/// calls; the audio thread never sees it.
struct App(Mutex<Result<audio::Engine, String>>);

#[derive(Serialize)]
struct Info {
    ok: bool,
    error: String,
    device: String,
    sample_rate: u32,
    channels: u16,
    lanes: Vec<u16>,
    bpm: f32,
}

#[derive(Serialize)]
struct Status {
    playing: bool,
    phase: f32,
    latency_ms: f32,
    xruns: u32,
}

#[tauri::command]
fn info(app: State<App>) -> Info {
    match &*app.0.lock().unwrap() {
        Ok(e) => Info {
            ok: true,
            error: String::new(),
            device: e.info.device.clone(),
            sample_rate: e.info.sample_rate,
            channels: e.info.channels,
            lanes: e.shared.lanes.iter().map(|l| l.load(Ordering::Relaxed)).collect(),
            bpm: e.shared.bpm(),
        },
        Err(err) => Info {
            ok: false,
            error: err.clone(),
            device: String::new(),
            sample_rate: 0,
            channels: 0,
            lanes: vec![],
            bpm: 0.0,
        },
    }
}

#[tauri::command]
fn status(app: State<App>) -> Option<Status> {
    let guard = app.0.lock().unwrap();
    let s = &guard.as_ref().ok()?.shared;
    Some(Status {
        playing: s.playing.load(Ordering::Relaxed),
        phase: s.phase(),
        latency_ms: s.latency_ms(),
        xruns: s.xruns.load(Ordering::Relaxed),
    })
}

fn with_engine<T>(app: &State<App>, f: impl FnOnce(&mut audio::Engine) -> T) -> Result<T, String> {
    match &mut *app.0.lock().unwrap() {
        Ok(e) => Ok(f(e)),
        Err(err) => Err(err.clone()),
    }
}

#[tauri::command]
fn set_playing(app: State<App>, playing: bool) -> Result<(), String> {
    with_engine(&app, |e| e.shared.playing.store(playing, Ordering::Relaxed))
}

#[tauri::command]
fn set_bpm(app: State<App>, bpm: f32) -> Result<(), String> {
    with_engine(&app, |e| e.shared.set_bpm(bpm))
}

#[tauri::command]
fn set_lane(app: State<App>, lane: usize, steps: u16) -> Result<(), String> {
    with_engine(&app, |e| {
        if let Some(l) = e.shared.lanes.get(lane) {
            l.store(steps, Ordering::Relaxed);
        }
    })
}

#[tauri::command]
fn hit(app: State<App>, lane: usize) -> Result<(), String> {
    with_engine(&app, |e| e.shared.hit(lane))
}

/// Opens the system picker, decodes the chosen WAV and hands it to the
/// sample lane. Async, so the picker and decoding stay off the UI thread.
#[tauri::command]
async fn pick_sample(handle: tauri::AppHandle, app: State<'_, App>) -> Result<String, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    handle
        .dialog()
        .file()
        .add_filter("Audio", &["wav", "WAV"])
        .pick_file(move |path| {
            let _ = tx.send(path);
        });
    let Some(path) = rx.recv().map_err(|e| e.to_string())? else {
        return Ok(String::new());
    };
    let name = path.to_string();
    let bytes = handle.fs().read(path).map_err(|e| format!("read {name}: {e}"))?;
    let rate = with_engine(&app, |e| e.info.sample_rate)?;
    let (samples, desc) = audio::decode_wav(&bytes, rate)?;
    with_engine(&app, |e| e.set_sample(samples))??;
    let short = name.rsplit(['/', '\\']).next().unwrap_or(&name).replace("%20", " ");
    Ok(format!("{short} - {desc}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        // In setup, not before: on Android the audio host needs the activity.
        .setup(|app| {
            tauri::Manager::manage(app, App(Mutex::new(audio::Engine::start())));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![info, status, set_playing, set_bpm, set_lane, hit, pick_sample])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
