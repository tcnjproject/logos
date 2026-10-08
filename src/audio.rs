// TCNJ AI/ML Group

//! Audio capture using cpal.
//!
//! `AudioCapture` owns the cpal stream on a background thread.
//! Samples are collected into a shared ring-buffer; the GUI polls it via a
//! Subscription that fires ~30 times/second and reads out the latest window
//! of samples as normalised f32 amplitudes for the waveform and VU meter.
//!
//! Additionally, when speech-to-text is active, the stream downmixes to mono,
//! resamples to 16 kHz, and streams 16-bit signed PCM (`i16`) chunks across
//! a crossbeam channel to the STT provider.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, SupportedStreamConfig};
use crossbeam_channel::Sender;
use std::sync::{Arc, Mutex};

/// Number of samples kept for the waveform display (covers ~20-30ms).
pub const WAVEFORM_LEN: usize = 1024;
pub const STT_TARGET_SAMPLE_RATE: u32 = 16_000;

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
    ///
    /// If `stt_sender` is `Some`, 16 kHz mono 16-bit signed PCM audio frames
    /// will be continuously streamed through the channel for speech recognition.
    pub fn start(stt_sender: Option<Sender<Vec<i16>>>) -> Result<Self, String> {
        let host = cpal::default_host();

        let device = host
            .default_input_device()
            .ok_or_else(|| "No default input device found".to_string())?;

        let supported_config: SupportedStreamConfig = device
            .default_input_config()
            .map_err(|e| format!("Could not get input config: {e}"))?;

        let channels = supported_config.channels() as usize;
        let sample_rate = supported_config.sample_rate().0;
        let config: cpal::StreamConfig = supported_config.clone().into();

        let shared = Arc::new(Mutex::new(AudioShared {
            samples: vec![0.0; WAVEFORM_LEN],
            rms: 0.0,
            peak: 0.0,
        }));

        let stream = match supported_config.sample_format() {
            SampleFormat::F32 => build_stream::<f32>(
                &device,
                &config,
                channels,
                sample_rate,
                shared.clone(),
                stt_sender,
            ),
            SampleFormat::I16 => build_stream::<i16>(
                &device,
                &config,
                channels,
                sample_rate,
                shared.clone(),
                stt_sender,
            ),
            SampleFormat::U16 => build_stream::<u16>(
                &device,
                &config,
                channels,
                sample_rate,
                shared.clone(),
                stt_sender,
            ),
            fmt => Err(format!("Unsupported sample format: {fmt:?}")),
        }?;

        stream
            .play()
            .map_err(|e| format!("Could not start audio stream: {e}"))?;

        Ok(Self {
            _stream: stream,
            shared,
        })
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    sample_rate: u32,
    shared: Arc<Mutex<AudioShared>>,
    stt_sender: Option<Sender<Vec<i16>>>,
) -> Result<Stream, String>
where
    T: cpal::Sample + cpal::SizedSample + ToF32,
{
    let err_fn = |e| eprintln!("Audio stream error: {e}");

    // Ring buffer accumulator for UI waveform
    let accumulator: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let acc_cb = accumulator.clone();
    let shared_cb = shared.clone();

    let stream = device
        .build_input_stream(
            config,
            move |data: &[T], _| {
                // 1. Convert all samples to normalized [-1.0, 1.0] floats
                let floats: Vec<f32> = data.iter().map(|s| s.to_f32()).collect();

                // 2. Downmix multi-channel to mono
                let mono: Vec<f32> = if channels > 1 {
                    floats
                        .chunks(channels)
                        .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
                        .collect()
                } else {
                    floats
                };

                // 3. Accumulate mono samples into UI ring buffer
                {
                    let mut acc = acc_cb.lock().unwrap();
                    acc.extend_from_slice(&mono);
                    let max = WAVEFORM_LEN * 4;
                    if acc.len() > max {
                        let drain = acc.len() - max;
                        acc.drain(..drain);
                    }
                }

                // 4. Compute RMS and Peak for VU meter
                let rms = {
                    let sum_sq: f32 = mono.iter().map(|s| s * s).sum();
                    (sum_sq / mono.len().max(1) as f32).sqrt()
                };
                let peak = mono.iter().map(|s| s.abs()).fold(0.0f32, f32::max);

                // Write to shared state
                {
                    let mut st = shared_cb.lock().unwrap();
                    let acc = acc_cb.lock().unwrap();
                    let start = acc.len().saturating_sub(WAVEFORM_LEN);
                    st.samples = acc[start..].to_vec();
                    while st.samples.len() < WAVEFORM_LEN {
                        st.samples.insert(0, 0.0);
                    }

                    st.rms = st.rms * 0.7 + rms * 0.3;
                    if peak > st.peak {
                        st.peak = peak;
                    } else {
                        st.peak *= 0.97;
                    }
                }

                // 5. If STT is active, resample mono f32 -> 16 kHz i16 and stream
                if let Some(ref tx) = stt_sender {
                    let resampled = resample_f32_to_16k_i16(&mono, sample_rate);
                    if !resampled.is_empty() {
                        let _ = tx.try_send(resampled);
                    }
                }
            },
            err_fn,
            None,
        )
        .map_err(|e| format!("Could not build input stream: {e}"))?;

    Ok(stream)
}

/// Convert and resample normalized mono f32 samples at `source_rate`
/// to 16 kHz 16-bit signed PCM (`i16`).
pub fn resample_f32_to_16k_i16(input: &[f32], source_rate: u32) -> Vec<i16> {
    if input.is_empty() {
        return Vec::new();
    }

    if source_rate == STT_TARGET_SAMPLE_RATE {
        return input
            .iter()
            .map(|&s| {
                let clamped = s.clamp(-1.0, 1.0);
                (clamped * i16::MAX as f32) as i16
            })
            .collect();
    }

    let ratio = source_rate as f64 / STT_TARGET_SAMPLE_RATE as f64;
    let out_len = ((input.len() as f64) / ratio).floor() as usize;
    let mut out = Vec::with_capacity(out_len);

    for i in 0..out_len {
        let src_pos = i as f64 * ratio;
        let idx = src_pos as usize;
        let frac = (src_pos - idx as f64) as f32;

        let sample = if idx + 1 < input.len() {
            let a = input[idx];
            let b = input[idx + 1];
            a + (b - a) * frac
        } else if idx < input.len() {
            input[idx]
        } else {
            0.0
        };

        let clamped = sample.clamp(-1.0, 1.0);
        let pcm = (clamped * i16::MAX as f32) as i16;
        out.push(pcm);
    }

    out
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resample_identity_rate() {
        let input = vec![0.0, 0.5, -0.5, 1.0, -1.0];
        let res = resample_f32_to_16k_i16(&input, 16_000);
        assert_eq!(res.len(), input.len());
        assert_eq!(res[0], 0);
        assert!((res[1] - (i16::MAX / 2)).abs() <= 2);
        assert!((res[2] - (-i16::MAX / 2)).abs() <= 2);
        assert_eq!(res[3], i16::MAX);
    }

    #[test]
    fn test_resample_48k_to_16k() {
        // 48 kHz -> 16 kHz is a 3:1 ratio
        let input: Vec<f32> = vec![0.5; 480];
        let res = resample_f32_to_16k_i16(&input, 48_000);
        assert_eq!(res.len(), 160);
        for sample in res {
            assert!((sample - (i16::MAX / 2)).abs() <= 2);
        }
    }
}
