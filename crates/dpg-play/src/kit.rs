//! The local kit: which files of the folder play which instrument. Laid
//! out for the dev test kit (the DP-2000 SSD5 render); edit [`FILES`] for
//! another.

use dpg_samples::{Kit, SampleFile, Sound};
use drum_patterns::Instrument;

/// Each instrument's normal sample and, if it has one, its ghost sample.
pub const FILES: [(Instrument, &str, Option<&str>); 4] = [
    (Instrument::Crash, "28_CRSH18_H.wav", None),
    (Instrument::HiHat, "17_HH_CL_H.wav", None),
    (Instrument::Snare, "03_SNR_H.wav", Some("06_SNR_GHST.wav")),
    (Instrument::Kick, "01_KICK_H.wav", None),
];

/// Loads the kit from `dir` at the output's `rate`.
pub fn load(dir: &str, rate: u32) -> Result<Kit, String> {
    let read = |name: &str| {
        let path = std::path::Path::new(dir).join(name);
        std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))
    };
    let mut kit = Kit::new(rate);
    for (instrument, normal, ghost) in FILES {
        let normal_bytes = read(normal)?;
        let ghost_bytes = ghost.map(read).transpose()?;
        let sound = Sound::load(wav(&normal_bytes), ghost_bytes.as_deref().map(wav), rate)
            .map_err(|e| format!("{normal}: {e}"))?;
        kit.set(instrument, sound);
    }
    Ok(kit)
}

fn wav(bytes: &[u8]) -> SampleFile<'_> {
    SampleFile {
        bytes,
        extension: Some("wav"),
    }
}
