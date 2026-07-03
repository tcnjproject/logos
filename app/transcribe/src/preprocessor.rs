//! Log-mel filterbank feature extraction matching NeMo's `AudioToMelSpectrogramPreprocessor`
//! (as implemented in `onnx_asr`'s `NemoPreprocessorNumpy`, which the Parakeet TDT ONNX export
//! expects as its input).
//!
//! Pipeline: pre-emphasis -> centered STFT (512-pt FFT, 400-sample Hann window, 160-sample hop)
//! -> 128-channel mel filterbank -> log -> per-utterance, per-mel-bin mean/variance
//! normalization across time.

use std::f32::consts::PI;
use std::sync::Arc;

use realfft::{num_complex::Complex32, RealFftPlanner, RealToComplex};

const N_FFT: usize = 512;
const WIN_LENGTH: usize = 400;
const HOP_LENGTH: usize = 160;
const N_MELS: usize = 128;
const N_FREQS: usize = N_FFT / 2 + 1;
const PREEMPH: f32 = 0.97;
const LOG_ZERO_GUARD: f32 = 1.0 / 16_777_216.0; // 2^-24

/// Mel filterbank matrix (257 FFT bins x 128 mel channels), extracted verbatim from
/// `onnx_asr`'s bundled `nemo128` filterbank so our output matches the reference implementation.
static FBANKS_BYTES: &[u8] = include_bytes!("../assets/nemo128_fbanks.bin");

pub struct FeatureExtractor {
    fft: Arc<dyn RealToComplex<f32>>,
    window: [f32; N_FFT],
    fbanks: Vec<f32>, // row-major [N_FREQS][N_MELS]
}

/// Extracted features in `(mel_channel, time_frame)` (channels-first) layout, as expected by the
/// Parakeet encoder's `audio_signal` input.
pub struct Features {
    /// `N_MELS * n_frames` values, channels-first.
    pub data: Vec<f32>,
    /// Total time frames in `data` (includes one trailing frame beyond `valid_frames` that NeMo's
    /// preprocessor always emits zeroed-out; the encoder still consumes it as part of the tensor).
    pub n_frames: usize,
    /// Frame count to pass as the encoder's `length` input (`waveform_len / hop_length`).
    pub valid_frames: usize,
}

impl FeatureExtractor {
    pub fn new() -> Self {
        let mut planner = RealFftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(N_FFT);

        // np.hanning(400), zero-padded to 512 samples, centered.
        let mut window = [0.0f32; N_FFT];
        let pad = (N_FFT - WIN_LENGTH) / 2;
        for n in 0..WIN_LENGTH {
            window[pad + n] = 0.5 - 0.5 * (2.0 * PI * n as f32 / (WIN_LENGTH - 1) as f32).cos();
        }

        let fbanks: Vec<f32> = FBANKS_BYTES
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();
        debug_assert_eq!(fbanks.len(), N_FREQS * N_MELS);

        Self { fft, window, fbanks }
    }

    /// Convert a mono 16kHz f32 waveform into normalized log-mel features.
    pub fn extract(&self, waveform: &[f32]) -> Features {
        let len = waveform.len();

        // Pre-emphasis: y[n] = x[n] - 0.97 * x[n-1] (x[-1] = 0).
        let mut emphasized = Vec::with_capacity(len);
        let mut prev = 0.0f32;
        for &sample in waveform {
            emphasized.push(sample - PREEMPH * prev);
            prev = sample;
        }

        // Zero-pad both sides by N_FFT/2 so frames are centered on the signal, matching
        // `np.pad(waveforms, (n_fft//2, n_fft//2))`.
        let pad = N_FFT / 2;
        let mut padded = vec![0.0f32; emphasized.len() + 2 * pad];
        padded[pad..pad + emphasized.len()].copy_from_slice(&emphasized);

        let n_frames = if padded.len() >= N_FFT {
            (padded.len() - N_FFT) / HOP_LENGTH + 1
        } else {
            0
        };

        let mut log_mel = vec![0.0f32; n_frames * N_MELS]; // [frame][mel]
        let mut fft_input = self.fft.make_input_vec();
        let mut fft_output = self.fft.make_output_vec();

        for frame in 0..n_frames {
            let start = frame * HOP_LENGTH;
            for i in 0..N_FFT {
                fft_input[i] = padded[start + i] * self.window[i];
            }
            self.fft
                .process(&mut fft_input, &mut fft_output)
                .expect("FFT input/output buffers have planner-provided lengths");

            let power: Vec<f32> = fft_output.iter().map(Complex32::norm_sqr).collect();

            for mel in 0..N_MELS {
                let mut acc = 0.0f32;
                for (freq, &p) in power.iter().enumerate() {
                    acc += p * self.fbanks[freq * N_MELS + mel];
                }
                log_mel[frame * N_MELS + mel] = (acc + LOG_ZERO_GUARD).ln();
            }
        }

        // NeMo derives the "valid" length from the original (unpadded) waveform length, which is
        // always one frame short of `n_frames`; mean/var are computed only over valid frames and
        // everything at or beyond `valid_frames` is zeroed rather than truncated.
        let valid_frames = (len / HOP_LENGTH).min(n_frames);

        // Per-utterance, per-mel-bin normalization across the time axis (NeMo's "per_feature"
        // normalization), using the same Bessel-corrected (n - 1) variance as the reference.
        let mut data = vec![0.0f32; N_MELS * n_frames]; // channels-first: [mel][frame]
        if valid_frames > 0 {
            for mel in 0..N_MELS {
                let mut mean = 0.0f32;
                for frame in 0..valid_frames {
                    mean += log_mel[frame * N_MELS + mel];
                }
                mean /= valid_frames as f32;

                let mut var = 0.0f32;
                for frame in 0..valid_frames {
                    let d = log_mel[frame * N_MELS + mel] - mean;
                    var += d * d;
                }
                var = if valid_frames > 1 {
                    var / (valid_frames - 1) as f32
                } else {
                    0.0
                };
                let denom = var.sqrt() + 1e-5;

                for frame in 0..valid_frames {
                    data[mel * n_frames + frame] = (log_mel[frame * N_MELS + mel] - mean) / denom;
                }
            }
        }

        Features {
            data,
            n_frames,
            valid_frames,
        }
    }
}

impl Default for FeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}
