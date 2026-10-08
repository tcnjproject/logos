//! Rhema voice activity detection; microphone capture remains owned by Logos.
pub mod meter;
pub mod types;
pub mod vad;
pub use types::AudioFrame;
pub use vad::{Vad, VadConfig, VadTransition};
