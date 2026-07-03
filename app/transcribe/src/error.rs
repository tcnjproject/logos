//! Error types for the `parakeet_transcribe` crate.

use std::path::PathBuf;

/// Errors that can occur while loading the model or transcribing audio.
#[derive(Debug, thiserror::Error)]
pub enum TranscribeError {
    #[error("ONNX Runtime error: {0}")]
    Onnx(#[from] ort::Error),

    #[error("failed to read model file {path}: {source}")]
    ModelFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("malformed vocabulary entry on line {line}: {text:?}")]
    Vocab { line: usize, text: String },

    #[error("no default input (microphone) device was found")]
    NoInputDevice,

    #[error("could not query microphone configurations: {0}")]
    DeviceConfig(#[from] cpal::SupportedStreamConfigsError),

    #[error("could not build input audio stream: {0}")]
    BuildStream(#[from] cpal::BuildStreamError),

    #[error("could not start audio stream: {0}")]
    PlayStream(#[from] cpal::PlayStreamError),

    #[error("unsupported microphone sample format: {0:?}")]
    UnsupportedSampleFormat(cpal::SampleFormat),

    #[error("audio worker thread terminated unexpectedly")]
    WorkerDied,

    #[error("failed to download model from Hugging Face: {0}")]
    Download(#[from] hf_hub::api::sync::ApiError),
}

// `SessionBuilder` methods return `ort::Error<SessionBuilder>` (parameterized by a "recover" type
// so the builder can be reclaimed after a failed call) rather than the plain `ort::Error` our
// `Onnx` variant converts from, so `?` needs an explicit bridge between the two.
impl From<ort::Error<ort::session::builder::SessionBuilder>> for TranscribeError {
    fn from(e: ort::Error<ort::session::builder::SessionBuilder>) -> Self {
        TranscribeError::Onnx(e.into())
    }
}

pub type Result<T> = std::result::Result<T, TranscribeError>;
