//! The Parakeet TDT (Token-and-Duration Transducer) acoustic model: a Conformer encoder plus an
//! LSTM decoder/joiner, run via ONNX Runtime, with greedy TDT decoding.
//!
//! This mirrors `onnx_asr`'s `NemoConformerTdt` model (see the `nemo-parakeet-tdt-0.6b-v3` config
//! shipped alongside the ONNX weights: 128 mel features in, 8x time subsampling, a 2-layer LSTM
//! decoder with 640 hidden units, and 5 duration bins).

use std::path::Path;

use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::Tensor;

use crate::error::Result;
use crate::preprocessor::FeatureExtractor;
use crate::vocab::Vocab;

const HIDDEN_SIZE: usize = 640;
const LSTM_LAYERS: usize = 2;
/// Greedy TDT decoding refuses to emit more than this many tokens for a single encoder frame,
/// guarding against runaway loops on corrupt/degenerate input (mirrors `onnx_asr`'s
/// `max_tokens_per_step` default).
const MAX_TOKENS_PER_STEP: usize = 10;

/// A loaded Parakeet TDT model, ready to transcribe 16kHz mono f32 audio.
pub struct Transcriber {
    encoder: Session,
    decoder_joint: Session,
    vocab: Vocab,
    features: FeatureExtractor,
}

/// LSTM decoder state: two `(LSTM_LAYERS, 1, HIDDEN_SIZE)` tensors (hidden and cell state).
struct DecoderState {
    h: Vec<f32>,
    c: Vec<f32>,
}

impl DecoderState {
    fn zeros() -> Self {
        let n = LSTM_LAYERS * HIDDEN_SIZE;
        Self {
            h: vec![0.0; n],
            c: vec![0.0; n],
        }
    }
}

impl Transcriber {
    /// Load the encoder, decoder/joint, and vocabulary from a directory containing
    /// `encoder-model.int8.onnx`, `decoder_joint-model.int8.onnx`, and `vocab.txt` (the layout
    /// `onnx_asr`/Hugging Face's `istupakov/parakeet-tdt-0.6b-v3-onnx` snapshots use).
    pub fn from_model_dir(dir: impl AsRef<Path>) -> Result<Self> {
        let dir = dir.as_ref();
        let build = |name: &str| -> Result<Session> {
            Ok(Session::builder()?
                .with_execution_providers([
                    ort::ep::TensorRT::default().build(),
                    ort::ep::CUDA::default().build(),
                    ort::ep::CPU::default().build(),
                ])?
                .with_optimization_level(GraphOptimizationLevel::All)?
                .with_intra_threads(4)?
                .with_inter_threads(1)?
                .commit_from_file(dir.join(name))?)
        };

        let encoder = build("encoder-model.int8.onnx")?;
        let decoder_joint = build("decoder_joint-model.int8.onnx")?;
        let vocab = Vocab::load(&dir.join("vocab.txt"))?;

        Ok(Self {
            encoder,
            decoder_joint,
            vocab,
            features: FeatureExtractor::new(),
        })
    }

    /// Transcribe a mono 16kHz f32 waveform (samples in `[-1.0, 1.0]`) into text.
    pub fn transcribe(&mut self, waveform: &[f32]) -> Result<String> {
        if waveform.is_empty() {
            return Ok(String::new());
        }

        let feats = self.features.extract(waveform);
        let (encoder_out, encoded_len) = self.run_encoder(&feats)?;
        let tokens = self.greedy_decode(&encoder_out, encoded_len)?;
        Ok(self.vocab.decode(&tokens))
    }

    /// Run the Conformer encoder, returning `(time_major_encoder_out, encoded_len)` where
    /// `time_major_encoder_out` is `encoded_len * 1024` values laid out as `[frame][channel]`.
    fn run_encoder(&mut self, feats: &crate::preprocessor::Features) -> Result<(Vec<f32>, usize)> {
        let audio_signal = Tensor::from_array((
            vec![1i64, 128, feats.n_frames as i64],
            feats.data.clone(),
        ))?;
        let length = Tensor::from_array((vec![1i64], vec![feats.valid_frames as i64]))?;

        let outputs = self.encoder.run(ort::inputs! {
            "audio_signal" => audio_signal,
            "length" => length,
        })?;

        let (out_shape, out_data) = outputs["outputs"].try_extract_tensor::<f32>()?;
        let (_, len_data) = outputs["encoded_lengths"].try_extract_tensor::<i64>()?;

        // `outputs` is (batch=1, 1024, T); transpose to time-major (T, 1024) for per-frame slicing.
        let channels = out_shape[1] as usize;
        let time = out_shape[2] as usize;
        let mut time_major = vec![0.0f32; channels * time];
        for c in 0..channels {
            for t in 0..time {
                time_major[t * channels + c] = out_data[c * time + t];
            }
        }

        Ok((time_major, len_data[0] as usize))
    }

    /// Greedy TDT decoding: at each encoder frame, repeatedly query the decoder/joiner for the
    /// next token (or blank) and a duration (0..=4 frames to advance), stopping either on blank,
    /// a nonzero duration, or after `MAX_TOKENS_PER_STEP` tokens without advancing (a safety valve
    /// against pathological inputs).
    fn greedy_decode(&mut self, encoder_out: &[f32], encoded_len: usize) -> Result<Vec<i32>> {
        let channels = 1024;
        let blank_idx = self.vocab.blank_idx as i32;

        let mut tokens: Vec<i32> = Vec::new();
        let mut state = DecoderState::zeros();
        let mut t = 0usize;
        let mut emitted_this_frame = 0usize;

        while t < encoded_len {
            let frame = &encoder_out[t * channels..(t + 1) * channels];
            let prev_token = tokens.last().copied().unwrap_or(blank_idx);

            let (token_logits, duration_step, new_state) =
                self.decode_step(prev_token, &state, frame)?;

            let token = argmax(&token_logits) as i32;

            if token != blank_idx {
                state = new_state;
                tokens.push(token);
                emitted_this_frame += 1;
            }

            if duration_step > 0 {
                t += duration_step;
                emitted_this_frame = 0;
            } else if token == blank_idx || emitted_this_frame == MAX_TOKENS_PER_STEP {
                t += 1;
                emitted_this_frame = 0;
            }
        }

        Ok(tokens)
    }

    /// One decoder/joiner step. Returns `(token_logits[vocab_size], duration_step, new_state)`.
    fn decode_step(
        &mut self,
        prev_token: i32,
        state: &DecoderState,
        encoder_frame: &[f32],
    ) -> Result<(Vec<f32>, usize, DecoderState)> {
        let channels = encoder_frame.len();
        let encoder_outputs = Tensor::from_array((
            vec![1i64, channels as i64, 1i64],
            encoder_frame.to_vec(),
        ))?;
        let targets = Tensor::from_array((vec![1i64, 1i64], vec![prev_token]))?;
        let target_length = Tensor::from_array((vec![1i64], vec![1i32]))?;
        let input_states_1 = Tensor::from_array((
            vec![LSTM_LAYERS as i64, 1i64, HIDDEN_SIZE as i64],
            state.h.clone(),
        ))?;
        let input_states_2 = Tensor::from_array((
            vec![LSTM_LAYERS as i64, 1i64, HIDDEN_SIZE as i64],
            state.c.clone(),
        ))?;

        let outputs = self.decoder_joint.run(ort::inputs! {
            "encoder_outputs" => encoder_outputs,
            "targets" => targets,
            "target_length" => target_length,
            "input_states_1" => input_states_1,
            "input_states_2" => input_states_2,
        })?;

        let (_, out_data) = outputs["outputs"].try_extract_tensor::<f32>()?;
        let (_, h_data) = outputs["output_states_1"].try_extract_tensor::<f32>()?;
        let (_, c_data) = outputs["output_states_2"].try_extract_tensor::<f32>()?;

        let vocab_size = self.vocab.vocab_size();
        let token_logits = out_data[..vocab_size].to_vec();
        let duration_logits = &out_data[vocab_size..];
        let duration_step = argmax(duration_logits);

        Ok((
            token_logits,
            duration_step,
            DecoderState {
                h: h_data.to_vec(),
                c: c_data.to_vec(),
            },
        ))
    }
}

fn argmax(values: &[f32]) -> usize {
    values
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(i, _)| i)
        .unwrap_or(0)
}
