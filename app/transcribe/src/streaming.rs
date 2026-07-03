//! Turns a raw stream of [`AudioChunk`]s from [`crate::mic`] into a running, line-by-line
//! transcript, mirroring the original Python prototype's loop: accumulate audio for the current
//! phrase, re-transcribe the accumulated buffer on every new chunk, and start a new line once the
//! microphone has been silent for longer than `phrase_timeout`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use crate::error::Result;
use crate::mic::AudioChunk;
use crate::model::Transcriber;

/// How often the receive loop polls for new audio while idle.
const POLL_INTERVAL: Duration = Duration::from_millis(250);

pub struct StreamConfig {
    /// Root-mean-square amplitude (in `[0.0, 1.0]`) a chunk must exceed to be treated as speech
    /// rather than silence. Chunks below this are dropped so that silence doesn't keep the
    /// current phrase alive indefinitely.
    pub energy_threshold: f32,
    /// How long the microphone must go quiet before the next chunk of speech starts a new
    /// transcript line instead of continuing the current one.
    pub phrase_timeout: Duration,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            energy_threshold: 0.02,
            phrase_timeout: Duration::from_secs(3),
        }
    }
}

/// Consume `rx` and drive `transcriber`, calling `on_update` with the full running transcript
/// (one line per phrase) every time it changes. Returns when the channel disconnects (i.e. the
/// [`crate::mic::MicStream`] is dropped) or `should_stop` is set (e.g. from a Ctrl+C handler).
pub fn run(
    transcriber: &mut Transcriber,
    rx: &Receiver<AudioChunk>,
    config: &StreamConfig,
    should_stop: &AtomicBool,
    mut on_update: impl FnMut(&[String]),
) -> Result<()> {
    let mut transcription: Vec<String> = vec![String::new()];
    let mut phrase_buffer: Vec<f32> = Vec::new();
    let mut last_speech_at: Option<Instant> = None;

    loop {
        if should_stop.load(Ordering::Relaxed) {
            return Ok(());
        }

        match rx.recv_timeout(POLL_INTERVAL) {
            Ok(chunk) => {
                if rms(&chunk.samples) < config.energy_threshold {
                    continue;
                }

                let phrase_complete = last_speech_at
                    .map(|t| chunk.captured_at.saturating_duration_since(t) > config.phrase_timeout)
                    .unwrap_or(false);
                if phrase_complete {
                    phrase_buffer.clear();
                    transcription.push(String::new());
                }
                last_speech_at = Some(chunk.captured_at);

                phrase_buffer.extend_from_slice(&chunk.samples);

                let text = transcriber.transcribe(&phrase_buffer)?;
                *transcription.last_mut().expect("always has at least one line") = text;
                on_update(&transcription);
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => return Ok(()),
        }
    }
}

fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}
