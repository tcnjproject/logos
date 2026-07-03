//! Ad-hoc validation tool: transcribe a raw little-endian f32 16kHz mono PCM file and print the
//! result, so it can be diffed against the Python `onnx_asr` reference on identical input.
//! Usage: cargo run --example compare -- <model_dir> <raw_f32_path>

use std::env;
use std::fs;

use parakeet_transcribe::Transcriber;

fn main() {
    let mut args = env::args().skip(1);
    let model_dir = args.next().expect("usage: compare <model_dir> <raw_f32_path>");
    let raw_path = args.next().expect("usage: compare <model_dir> <raw_f32_path>");

    let bytes = fs::read(&raw_path).expect("read raw f32 file");
    let samples: Vec<f32> = bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect();
    println!("loaded {} samples ({:.2}s)", samples.len(), samples.len() as f32 / 16000.0);

    let mut transcriber = Transcriber::from_model_dir(&model_dir).expect("load model");
    let text = transcriber.transcribe(&samples).expect("transcribe");
    println!("RUST TEXT: {text:?}");
}
