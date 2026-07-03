//! Microphone capture via [`cpal`](https://docs.rs/cpal), the standard cross-platform audio I/O
//! crate for Rust (already used elsewhere in this workspace for waveform capture). Captured audio
//! is downmixed to mono, resampled to the model's expected 16kHz (see [`crate::resample`] for the
//! anti-aliased resampler this uses), and delivered over a channel in fixed-duration windows —
//! segmentation into "phrases" (deciding when a pause should start a new transcript line) happens
//! one layer up, in [`crate::streaming`].

use std::sync::mpsc;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SupportedStreamConfig};

use crate::error::{Result, TranscribeError};
use crate::resample::Resampler;

/// Sample rate the acoustic model was trained on; microphone audio is resampled to this rate.
pub const TARGET_SAMPLE_RATE: u32 = 16_000;

/// Default cadence at which accumulated audio is handed off to the transcriber, matching the
/// original prototype's `--record_timeout` default.
pub const DEFAULT_CHUNK_DURATION: Duration = Duration::from_secs(2);

/// A chunk of mono 16kHz audio captured from the microphone.
pub struct AudioChunk {
    pub samples: Vec<f32>,
    pub captured_at: Instant,
}

/// Owns the live input stream; drop to stop capturing.
pub struct MicStream {
    _stream: cpal::Stream,
}

/// List the names of available input (microphone) devices, e.g. for a `--list-microphones` CLI flag.
pub fn list_input_devices() -> Result<Vec<String>> {
    let host = cpal::default_host();
    Ok(host
        .input_devices()
        .map_err(|_| TranscribeError::NoInputDevice)?
        .filter_map(|d| d.name().ok())
        .collect())
}

/// Open the default input device and start streaming mono 16kHz `f32` audio, handed off to the
/// returned receiver in `chunk_duration`-sized windows (audio is accumulated across driver
/// callbacks until a full window is ready).
pub fn start(chunk_duration: Duration) -> Result<(MicStream, mpsc::Receiver<AudioChunk>)> {
    let host = cpal::default_host();
    let device = host.default_input_device().ok_or(TranscribeError::NoInputDevice)?;
    start_on_device(&device, chunk_duration)
}

fn start_on_device(
    device: &cpal::Device,
    chunk_duration: Duration,
) -> Result<(MicStream, mpsc::Receiver<AudioChunk>)> {
    let config = pick_config(device)?;
    let sample_rate = config.sample_rate().0;
    let channels = config.channels() as usize;
    let sample_format = config.sample_format();
    let chunk_samples = ((chunk_duration.as_secs_f64() * TARGET_SAMPLE_RATE as f64).round() as usize).max(1);

    let (tx, rx) = mpsc::channel();
    let err_fn = |e| eprintln!("audio stream error: {e}");
    let mut accumulator = ChunkAccumulator::new(chunk_samples, Resampler::new(sample_rate, TARGET_SAMPLE_RATE));

    let stream = match sample_format {
        SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data: &[f32], _| accumulator.push(data, channels, &tx),
            err_fn,
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data: &[i16], _| {
                let floats: Vec<f32> = data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                accumulator.push(&floats, channels, &tx)
            },
            err_fn,
            None,
        ),
        SampleFormat::U16 => device.build_input_stream(
            &config.into(),
            move |data: &[u16], _| {
                let floats: Vec<f32> = data
                    .iter()
                    .map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0)
                    .collect();
                accumulator.push(&floats, channels, &tx)
            },
            err_fn,
            None,
        ),
        fmt => return Err(TranscribeError::UnsupportedSampleFormat(fmt)),
    }?;

    stream.play()?;

    Ok((MicStream { _stream: stream }, rx))
}

/// Buffers downmixed, resampled audio (owned solely by the audio callback thread) until it has
/// enough samples for one `chunk_duration` window, then hands the window off over the channel.
struct ChunkAccumulator {
    buffer: Vec<f32>,
    chunk_samples: usize,
    resampler: Resampler,
}

impl ChunkAccumulator {
    fn new(chunk_samples: usize, resampler: Resampler) -> Self {
        Self {
            buffer: Vec::with_capacity(chunk_samples),
            chunk_samples,
            resampler,
        }
    }

    fn push(&mut self, data: &[f32], channels: usize, tx: &mpsc::Sender<AudioChunk>) {
        if data.is_empty() {
            return;
        }
        let mono = downmix(data, channels);
        let resampled = self.resampler.process(&mono);
        self.buffer.extend_from_slice(&resampled);

        if self.buffer.len() >= self.chunk_samples {
            // The receiver is dropped once the app is shutting down; ignore send errors.
            let _ = tx.send(AudioChunk {
                samples: std::mem::take(&mut self.buffer),
                captured_at: Instant::now(),
            });
        }
    }
}

fn downmix(data: &[f32], channels: usize) -> Vec<f32> {
    if channels <= 1 {
        return data.to_vec();
    }
    data.chunks_exact(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect()
}

/// Prefer a config offering [`TARGET_SAMPLE_RATE`] directly (avoids resampling); otherwise fall
/// back to the device's default input config and resample in software.
fn pick_config(device: &cpal::Device) -> Result<SupportedStreamConfig> {
    let target = cpal::SampleRate(TARGET_SAMPLE_RATE);
    if let Ok(mut ranges) = device.supported_input_configs() {
        if let Some(range) = ranges.find(|r| r.min_sample_rate() <= target && target <= r.max_sample_rate()) {
            return Ok(range.with_sample_rate(target));
        }
    }
    device
        .default_input_config()
        .map_err(|_| TranscribeError::NoInputDevice)
}
