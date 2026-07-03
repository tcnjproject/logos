//! Real-time speech-to-text transcription using the Parakeet TDT 0.6B ONNX model, run through
//! ONNX Runtime (via the [`ort`] crate). This is a Rust port of the project's original Python
//! prototype (`transcribe.py`), split into three layers:
//!
//! - [`model`]: loads the encoder/decoder ONNX sessions and vocabulary, and runs greedy TDT
//!   decoding on a waveform (feature extraction lives in [`preprocessor`], detokenization in
//!   [`vocab`]).
//! - [`hub`]: downloads (or reuses a cached copy of) the model files directly from the Hugging
//!   Face Hub, so no separate Python step is required to populate the model directory.
//! - [`mic`]: captures microphone audio with [`cpal`](https://docs.rs/cpal) and resamples it to
//!   the model's expected 16kHz mono format.
//! - [`streaming`]: glues the two together into a running, line-by-line transcript, starting a
//!   new line after a configurable period of silence.
//!
//! See `src/main.rs` for a complete example wiring a live microphone to the transcriber.

pub mod error;
pub mod hub;
pub mod mic;
pub mod model;
pub mod preprocessor;
pub mod resample;
pub mod streaming;
pub mod vocab;

pub use error::{Result, TranscribeError};
pub use hub::{download_model, DEFAULT_MODEL_ID};
pub use mic::{list_input_devices, AudioChunk, MicStream, DEFAULT_CHUNK_DURATION, TARGET_SAMPLE_RATE};
pub use model::Transcriber;
pub use streaming::StreamConfig;
