// TCNJ AI/ML Group

//! Audio capture using cpal.
//!
//! `AudioCapture` owns the cpal stream on a background thread.
//! Samples are collected into a shared ring-buffer; the GUI polls it via a
//! Subscription that fires ~30 times/second and reads out the latest window
//! of samples as normalised f32 amplitudes for the waveform display.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, SampleFormat, SupportedStreamConfig};
use std::sync::{Arc, Mutex};

/// Number of samples kept for the waveform display (covers ~30ms at 48kHz).
pub const WAVEFORM_LEN: usize = 1024;

/// Shared audio state between the cpal callback thread and the GUI thread.
#[derive(Default)]
pub struct AudioShared {
    /// Latest `WAVEFORM_LEN` normalised samples in [-1.0, 1.0].
    pub samples: Vec<f32>,
    /// Root-mean-square level in [0.0, 1.0].
    pub rms: f32,
    /// Peak level in [0.0, 1.0] (decays over time).
    pub peak: f32,
}

/// Handle that keeps the cpal stream alive.
/// Drop to stop recording.
pub struct AudioCapture {
    _stream: Stream,
    pub shared: Arc<Mutex<AudioShared>>,
}

impl AudioCapture {
    /// Open the default input device and start streaming.
    /// Returns `Err` with a human-readable message on failure.
    pub fn start() -> Result<Self, String> {
        let host = cpal::default_host();

        let device = host
            .default_input_device()
            .ok_or_else(|| "No default input device found".to_string())?;

        let config: SupportedStreamConfig = device
            .default_input_config()
            .map_err(|e| format!("Could not get input config: {e}"))?;

        let shared = Arc::new(Mutex::new(AudioShared {
            samples: vec![0.0; WAVEFORM_LEN],
            rms: 0.0,
            peak: 0.0,
        }));

        let stream = match config.sample_format() {
            SampleFormat::F32 => build_stream::<f32>(&device, &config.into(), shared.clone()),
            SampleFormat::I16 => build_stream::<i16>(&device, &config.into(), shared.clone()),
            SampleFormat::U16 => build_stream::<u16>(&device, &config.into(), shared.clone()),
            fmt => Err(format!("Unsupported sample format: {fmt:?}")),
        }?;

        stream
            .play()
            .map_err(|e| format!("Could not start stream: {e}"))?;

        Ok(Self {
            _stream: stream,
            shared,
        })
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    shared: Arc<Mutex<AudioShared>>,
) -> Result<Stream, String>
where
    T: cpal::Sample + cpal::SizedSample + ToF32,
{
    let err_fn = |e| eprintln!("Audio stream error: {e}");

    // Ring buffer accumulator
    let accumulator: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let acc_cb = accumulator.clone();
    let shared_cb = shared.clone();

    let stream = device
        .build_input_stream(
            config,
            move |data: &[T], _| {
                let floats: Vec<f32> = data.iter().map(|s| s.to_f32()).collect();

                // Accumulate into ring buffer
                {
                    let mut acc = acc_cb.lock().unwrap();
                    acc.extend_from_slice(&floats);
                    // Keep only the last WAVEFORM_LEN * 4 samples to bound memory
                    let max = WAVEFORM_LEN * 4;
                    if acc.len() > max {
                        let drain = acc.len() - max;
                        acc.drain(..drain);
                    }
                }

                // Compute RMS and peak over this chunk
                let rms = {
                    let sum_sq: f32 = floats.iter().map(|s| s * s).sum();
                    (sum_sq / floats.len().max(1) as f32).sqrt()
                };
                let peak = floats.iter().map(|s| s.abs()).fold(0.0f32, f32::max);

                // Write to shared state
                {
                    let mut st = shared_cb.lock().unwrap();

                    // Copy latest WAVEFORM_LEN samples for the waveform view
                    let acc = acc_cb.lock().unwrap();
                    let start = acc.len().saturating_sub(WAVEFORM_LEN);
                    st.samples = acc[start..].to_vec();
                    // Pad to full length if not enough yet
                    while st.samples.len() < WAVEFORM_LEN {
                        st.samples.insert(0, 0.0);
                    }

                    // Smooth RMS with a simple low-pass filter
                    st.rms = st.rms * 0.7 + rms * 0.3;

                    // Peak holds the maximum, decays slowly
                    if peak > st.peak {
                        st.peak = peak;
                    } else {
                        st.peak *= 0.97; // decay
                    }
                }
            },
            err_fn,
            None,
        )
        .map_err(|e| format!("Could not build input stream: {e}"))?;

    Ok(stream)
}

/// Trait to normalise any cpal sample type to f32 in [-1.0, 1.0].
pub trait ToF32 {
    fn to_f32(self) -> f32;
}

impl ToF32 for f32 {
    fn to_f32(self) -> f32 {
        self.clamp(-1.0, 1.0)
    }
}

impl ToF32 for i16 {
    fn to_f32(self) -> f32 {
        self as f32 / i16::MAX as f32
    }
}

impl ToF32 for u16 {
    fn to_f32(self) -> f32 {
        (self as f32 / u16::MAX as f32) * 2.0 - 1.0
    }
}