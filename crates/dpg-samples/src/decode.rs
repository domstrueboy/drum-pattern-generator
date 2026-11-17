//! Decoding WAV, FLAC, MP3 and M4A (AAC) to mono `f32`, the file's own
//! sample rate kept.

use std::fmt;
use std::io::Cursor;

use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::errors::Error;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

#[cfg(test)]
#[path = "decode_tests.rs"]
mod tests;

/// The longest sound accepted. A crash rings out well inside it; a whole
/// song picked by mistake doesn't fit.
pub const MAX_SECONDS: u32 = 10;

/// Decoded mono audio, full scale at +/-1.0.
#[derive(Debug, Clone, PartialEq)]
pub struct Audio {
    /// Frames a second.
    pub rate: u32,
    /// One sample a frame; channels are averaged.
    pub samples: Vec<f32>,
}

/// Why a file couldn't be decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Not a format or codec this app reads.
    Unsupported(String),
    /// A format it reads, but damaged.
    Malformed(String),
    /// No audio in it.
    Empty,
    /// Longer than [`MAX_SECONDS`].
    TooLong,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported(why) => write!(f, "not a supported audio file ({why})"),
            Self::Malformed(why) => write!(f, "the file is damaged ({why})"),
            Self::Empty => write!(f, "the file has no audio"),
            Self::TooLong => write!(f, "longer than {MAX_SECONDS} s - is it a drum hit?"),
        }
    }
}

impl std::error::Error for DecodeError {}

impl From<Error> for DecodeError {
    fn from(error: Error) -> Self {
        match error {
            Error::Unsupported(why) => Self::Unsupported(why.into()),
            other => Self::Malformed(other.to_string()),
        }
    }
}

/// Decodes a whole file to mono. `extension` ("wav", "m4a"...) helps pick
/// the reader when known; the contents decide either way.
pub fn decode(bytes: &[u8], extension: Option<&str>) -> Result<Audio, DecodeError> {
    let source = MediaSourceStream::new(Box::new(Cursor::new(bytes)), Default::default());
    let mut hint = Hint::new();
    if let Some(extension) = extension {
        hint.with_extension(extension);
    }
    let mut format = symphonia::default::get_probe().probe(
        &hint,
        source,
        FormatOptions::default(),
        MetadataOptions::default(),
    )?;
    let track = format
        .default_track(TrackType::Audio)
        .ok_or(DecodeError::Empty)?;
    let track_id = track.id;
    let params = track
        .codec_params
        .as_ref()
        .and_then(|p| p.audio())
        .ok_or(DecodeError::Empty)?;
    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(params, &AudioDecoderOptions::default())?;

    let mut rate = 0;
    let mut samples = Vec::new();
    let mut frame = Vec::new();
    loop {
        let packet = match format.next_packet() {
            Ok(Some(packet)) => packet,
            Ok(None) => break,
            // Some readers report the end of the stream as a cut-off read.
            Err(Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        };
        if packet.track_id != track_id {
            continue;
        }
        let buffer = match decoder.decode(&packet) {
            Ok(buffer) => buffer,
            // One bad packet is a click at worst; keep the rest.
            Err(Error::DecodeError(_)) => continue,
            Err(e) => return Err(e.into()),
        };
        let channels = buffer.spec().channels().count().max(1);
        rate = buffer.spec().rate();
        frame.resize(buffer.samples_interleaved(), 0.0f32);
        buffer.copy_to_slice_interleaved(&mut frame);
        samples.extend(
            frame
                .chunks_exact(channels)
                .map(|c| c.iter().sum::<f32>() / channels as f32),
        );
        if samples.len() > (MAX_SECONDS * rate) as usize {
            return Err(DecodeError::TooLong);
        }
    }

    if samples.is_empty() || rate == 0 {
        return Err(DecodeError::Empty);
    }
    Ok(Audio { rate, samples })
}
