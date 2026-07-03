//! Streaming sample-rate conversion for microphone audio.
//!
//! Most audio devices (Windows WASAPI shared-mode devices in particular) only expose a single
//! fixed native rate — commonly 48kHz — rather than the model's expected 16kHz, so capture
//! generally has to downsample by a non-trivial ratio (e.g. 3:1). Naively decimating (or linearly
//! interpolating) a signal without first band-limiting it aliases everything above the new
//! Nyquist frequency back down into the audible range; for speech that's exactly where sibilants
//! (*s*, *f*, *sh*) live, so the aliasing shows up as corrupted consonants and, in turn, words the
//! acoustic model mishears. [`Resampler`] applies a windowed-sinc lowpass filter before decimation
//! to avoid that, carrying filter and interpolation state across successive [`Resampler::process`]
//! calls so chunk boundaries (driver callbacks can hand over buffers of any size) introduce no
//! discontinuities.

use std::f64::consts::PI;

/// Taps in the anti-aliasing lowpass filter. Must be odd (Type I linear-phase FIR) so the filter
/// has an exact integer group delay. 129 taps gives a transition band of roughly 1.2kHz around an
/// 8kHz cutoff at a 48kHz input rate — tight enough to protect sibilants while keeping the filter
/// (and the latency its group delay adds) cheap enough for a real-time audio callback.
const FILTER_TAPS: usize = 129;

/// Converts a stream of audio chunks from `from_rate` to `to_rate`, band-limiting before
/// decimation to avoid aliasing when downsampling.
pub struct Resampler {
    from_rate: u32,
    to_rate: u32,
    /// Anti-aliasing FIR taps; empty when upsampling or rates match (no aliasing risk, so no
    /// filtering is needed).
    taps: Vec<f32>,
    /// The last `taps.len() - 1` raw input samples, carried across calls so the filter has full
    /// context right at the start of each new chunk.
    filter_history: Vec<f32>,
    /// Filtered samples not yet consumed by decimation, carried across calls.
    pending: Vec<f32>,
    /// Fractional read position into `pending`, in samples.
    pos: f64,
}

impl Resampler {
    pub fn new(from_rate: u32, to_rate: u32) -> Self {
        let taps = if from_rate > to_rate {
            design_lowpass(from_rate, to_rate, FILTER_TAPS)
        } else {
            Vec::new()
        };
        let filter_history = vec![0.0; taps.len().saturating_sub(1)];
        Self {
            from_rate,
            to_rate,
            taps,
            filter_history,
            pending: Vec::new(),
            pos: 0.0,
        }
    }

    /// Resample `input` (at `from_rate`), returning as many output samples (at `to_rate`) as are
    /// now available. Some input is always retained internally to keep interpolation continuous
    /// across calls, so callers should not expect a fixed input:output ratio per call.
    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        if self.from_rate == self.to_rate {
            return input.to_vec();
        }
        if input.is_empty() {
            return Vec::new();
        }

        let filtered = if self.taps.is_empty() {
            input.to_vec()
        } else {
            self.filter(input)
        };
        self.pending.extend_from_slice(&filtered);

        let ratio = self.from_rate as f64 / self.to_rate as f64;
        let mut out = Vec::new();
        while (self.pos.floor() as usize) + 1 < self.pending.len() {
            let idx = self.pos.floor() as usize;
            let frac = (self.pos - idx as f64) as f32;
            let a = self.pending[idx];
            let b = self.pending[idx + 1];
            out.push(a + (b - a) * frac);
            self.pos += ratio;
        }

        // Drop the now-fully-consumed prefix, keeping the sample at `floor(pos)` since the next
        // call's first interpolation still needs it as its left endpoint. `pos` can overshoot
        // past the end of `pending` (the loop's last `pos += ratio` isn't re-checked against the
        // bound), so the drain count must be capped at `pending.len()` — and that same capped
        // count, not the uncapped `floor(pos)`, is what `pos` must be rebased against, or `pos`
        // goes negative and desyncs decimation from the next call onward.
        let consumed = (self.pos.floor() as usize).min(self.pending.len());
        if consumed > 0 {
            self.pending.drain(..consumed);
            self.pos -= consumed as f64;
        }

        out
    }

    /// Convolve `input` with the anti-aliasing filter, using `filter_history` for continuity
    /// across chunk boundaries.
    fn filter(&mut self, input: &[f32]) -> Vec<f32> {
        let n = self.taps.len();
        let mut extended = Vec::with_capacity(self.filter_history.len() + input.len());
        extended.extend_from_slice(&self.filter_history);
        extended.extend_from_slice(input);

        let mut out = Vec::with_capacity(input.len());
        for i in 0..input.len() {
            let end = n - 1 + i; // index into `extended` of the sample this output corresponds to
            let mut acc = 0.0f32;
            for (k, &h) in self.taps.iter().enumerate() {
                acc += h * extended[end - k];
            }
            out.push(acc);
        }

        let keep_from = extended.len().saturating_sub(n - 1);
        self.filter_history = extended[keep_from..].to_vec();

        out
    }
}

/// Windowed-sinc lowpass filter design (Hamming window), with unity DC gain. `cutoff` is derived
/// from the target Nyquist frequency with a small backoff, leaving headroom for the transition
/// band so content just below the target Nyquist isn't attenuated.
fn design_lowpass(from_rate: u32, to_rate: u32, num_taps: usize) -> Vec<f32> {
    debug_assert!(num_taps % 2 == 1, "filter should have an odd number of taps for linear phase");

    let cutoff = 0.9 * (to_rate as f64 / 2.0) / from_rate as f64; // normalized to [0, 0.5]
    let m = (num_taps - 1) as f64;

    let mut taps = vec![0.0f64; num_taps];
    for (n, tap) in taps.iter_mut().enumerate() {
        let x = n as f64 - m / 2.0;
        let sinc = if x == 0.0 {
            2.0 * cutoff
        } else {
            (2.0 * PI * cutoff * x).sin() / (PI * x)
        };
        let window = 0.54 - 0.46 * (2.0 * PI * n as f64 / m).cos(); // Hamming
        *tap = sinc * window;
    }

    let dc_gain: f64 = taps.iter().sum();
    taps.iter().map(|&t| (t / dc_gain) as f32).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(freq: f64, rate: u32, n: usize) -> Vec<f32> {
        (0..n).map(|i| (2.0 * PI * freq * i as f64 / rate as f64).sin() as f32).collect()
    }

    fn rms(samples: &[f32]) -> f32 {
        (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt()
    }

    /// Feeding chunks of varying, non-uniform sizes should give the same result as one big push,
    /// proving state carries correctly across chunk boundaries (the shape driver callbacks take).
    #[test]
    fn chunking_does_not_change_output() {
        let signal = sine(440.0, 48_000, 48_000 * 2);

        let mut whole = Resampler::new(48_000, 16_000);
        let all_at_once = whole.process(&signal);

        let mut chunked = Resampler::new(48_000, 16_000);
        let mut piecewise = Vec::new();
        for chunk in signal.chunks(377) {
            piecewise.extend(chunked.process(chunk));
        }

        assert_eq!(all_at_once.len(), piecewise.len());
        for (a, b) in all_at_once.iter().zip(piecewise.iter()) {
            assert!((a - b).abs() < 1e-4, "{a} vs {b}");
        }
    }

    /// A tone above the target Nyquist (e.g. 12kHz, downsampling 48kHz -> 16kHz) must not survive
    /// into the resampled output with significant energy, or it aliases into an audible false
    /// frequency and corrupts speech (this is the bug being fixed).
    #[test]
    fn attenuates_frequencies_above_target_nyquist() {
        let signal = sine(12_000.0, 48_000, 48_000 * 2);
        let mut resampler = Resampler::new(48_000, 16_000);
        let out = resampler.process(&signal);

        // Drop the filter's startup transient (its group delay) before measuring steady state.
        let steady = &out[500..];
        assert!(rms(steady) < 0.05, "12kHz tone leaked through with rms {}", rms(steady));
    }

    /// A tone well within the passband should survive resampling near full amplitude, so the
    /// anti-aliasing filter isn't muffling legitimate speech content.
    #[test]
    fn preserves_frequencies_within_passband() {
        let signal = sine(1_000.0, 48_000, 48_000 * 2);
        let mut resampler = Resampler::new(48_000, 16_000);
        let out = resampler.process(&signal);

        let steady = &out[500..];
        let expected_rms = 1.0 / std::f32::consts::SQRT_2;
        assert!(
            (rms(steady) - expected_rms).abs() < 0.05,
            "1kHz tone was attenuated: rms {} (expected ~{expected_rms})",
            rms(steady)
        );
    }

    #[test]
    fn matching_rates_pass_through_unchanged() {
        let signal = sine(440.0, 16_000, 1_000);
        let mut resampler = Resampler::new(16_000, 16_000);
        assert_eq!(resampler.process(&signal), signal);
    }
}
