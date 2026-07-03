# parakeet-transcribe

Real-time microphone speech-to-text using the [Parakeet TDT 0.6B v3](https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx)
ONNX model, run through [ONNX Runtime](https://onnxruntime.ai/) via the [`ort`](https://docs.rs/ort) crate.

This is a Rust port of the project's original Python prototype (`app/backend/transcribe.py`), split
into a reusable library (`parakeet_transcribe`) and a small CLI binary (`transcribe`) that captures
the default microphone and prints a live, line-by-line transcript.

## Layout

| Module | Responsibility |
| --- | --- |
| [`src/preprocessor.rs`](src/preprocessor.rs) | Log-mel filterbank feature extraction (STFT + mel filterbank + per-utterance normalization), matching NeMo's `AudioToMelSpectrogramPreprocessor`. |
| [`assets/nemo128_fbanks.bin`](assets/nemo128_fbanks.bin) | The mel filterbank matrix itself (257 FFT bins x 128 mel channels, raw little-endian `f32`), extracted verbatim from `onnx_asr`'s bundled `nemo128` filterbank and embedded via `include_bytes!` so the STFT-to-mel projection matches the Python reference bit-for-bit rather than being recomputed from a formula. |
| [`src/model.rs`](src/model.rs) | Loads the Conformer encoder and LSTM decoder/joiner ONNX sessions and runs greedy TDT (token-and-duration transducer) decoding. |
| [`src/vocab.rs`](src/vocab.rs) | Vocabulary loading and SentencePiece-style detokenization. |
| [`src/mic.rs`](src/mic.rs) | Microphone capture via [`cpal`](https://docs.rs/cpal), with downmixing to mono and resampling to 16kHz. |
| [`src/streaming.rs`](src/streaming.rs) | Turns a raw audio stream into a running transcript: buffers the current phrase and starts a new line after a period of silence. |
| [`src/main.rs`](src/main.rs) | CLI entry point wiring the above together. |

The feature extraction, encoder/decoder I/O, and TDT decoding loop were verified against the
Python [`onnx_asr`](https://github.com/istupakov/onnx-asr) reference implementation on identical
audio input and produce byte-for-byte identical transcripts.

## Model files

The crate expects a directory containing:

- `encoder-model.int8.onnx`
- `decoder_joint-model.int8.onnx`
- `vocab.txt`
- `config.json`

By default it points at the Hugging Face snapshot already cached by the Python prototype:

```
app/backend/models/models--istupakov--parakeet-tdt-0.6b-v3-onnx/snapshots/<hash>
```

If that snapshot hash ever changes (e.g. after re-running the Python model loader) or you keep the
model somewhere else, override it with `--model-dir`.

## Usage

```sh
# List available microphone input devices.
cargo run --release --bin transcribe -- --list-microphones

# Transcribe from the default microphone using the default (cached) model directory.
cargo run --release --bin transcribe

# Point at a different model directory and tune phrase segmentation.
cargo run --release --bin transcribe -- \
  --model-dir /path/to/parakeet-tdt-0.6b-v3-onnx \
  --energy-threshold 0.02 \
  --record-timeout 2 \
  --phrase-timeout 3
```

Press Ctrl+C to stop; the final full transcript is printed before exit.

### CLI options

| Flag | Default | Meaning |
| --- | --- | --- |
| `--model-dir <DIR>` | cached snapshot under `app/backend/models` | Directory with the ONNX model files and vocab. |
| `--energy-threshold <F>` | `0.02` | RMS amplitude (0.0-1.0) a captured window must exceed to be treated as speech rather than silence. |
| `--record-timeout <SECS>` | `2` | How often captured audio is handed to the model — controls how "live" updates feel. |
| `--phrase-timeout <SECS>` | `3` | Seconds of silence before the next utterance starts a new transcript line. |
| `--list-microphones` | off | List input device names and exit. |

## Using the library directly

```rust
use parakeet_transcribe::{mic, streaming::{self, StreamConfig}, Transcriber};
use std::sync::atomic::AtomicBool;
use std::time::Duration;

let mut transcriber = Transcriber::from_model_dir("path/to/model_dir")?;
let (_mic_stream, rx) = mic::start(Duration::from_secs(2))?;
let should_stop = AtomicBool::new(false);

streaming::run(&mut transcriber, &rx, &StreamConfig::default(), &should_stop, |lines| {
    println!("{lines:?}");
})?;
```

`Transcriber::transcribe(&mut self, waveform: &[f32])` also works standalone on any mono 16kHz
`f32` buffer (e.g. loaded from a WAV file) if you don't need live microphone capture.

## GPU acceleration

CUDA and TensorRT execution providers are opt-in via Cargo features (they require an ONNX Runtime
build with the corresponding support):

```sh
cargo run --release --features cuda      # try CUDA, falling back to CPU
cargo run --release --features tensorrt  # try TensorRT, then CUDA, then CPU
```

Without either feature, inference runs on CPU only.

## Tests and validation

```sh
cargo test              # unit tests (vocab detokenization)
cargo clippy --all-targets
```

`examples/compare.rs` transcribes a raw little-endian `f32` 16kHz mono PCM file, useful for
diffing against the Python reference on identical input without needing a live microphone:

```sh
cargo run --release --example compare -- <model_dir> <raw_f32_path>
```
