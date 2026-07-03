//! CLI front-end: captures the default microphone and prints a live transcript, using the
//! `parakeet_transcribe` library crate for everything (model loading, feature extraction, TDT
//! decoding, and audio capture). This is the Rust equivalent of the project's original
//! `transcribe.py` prototype.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;

use parakeet_transcribe::{mic, streaming, Transcriber};

/// Default model directory: the `istupakov/parakeet-tdt-0.6b-v3-onnx` snapshot already cached by
/// the Python prototype (`app/backend/transcribe.py`) under its Hugging Face hub cache, resolved
/// relative to this crate's directory (`app/transcribe`). Override with `--model-dir` if your
/// snapshot hash differs or the model lives elsewhere.
const DEFAULT_MODEL_DIR: &str =
    "models/models--istupakov--parakeet-tdt-0.6b-v3-onnx/snapshots/8f23f0c03c8761650bdb5b40aaf3e40d2c15f1ce";

/// Real-time microphone transcription using the Parakeet TDT 0.6B ONNX model.
#[derive(Parser)]
struct Args {
    /// Directory containing `encoder-model.int8.onnx`, `decoder_joint-model.int8.onnx`,
    /// `vocab.txt`, and `config.json`.
    #[arg(long, value_name = "DIR", default_value = DEFAULT_MODEL_DIR)]
    model_dir: PathBuf,

    /// RMS energy level (0.0-1.0) a captured window must exceed to be treated as speech rather
    /// than silence.
    #[arg(long, default_value_t = 0.02)]
    energy_threshold: f32,

    /// How often, in seconds, captured audio is handed to the model (how "live" updates are).
    #[arg(long, default_value_t = 2.0)]
    record_timeout: f32,

    /// Seconds of silence before the next utterance starts a new transcript line.
    #[arg(long, default_value_t = 3.0)]
    phrase_timeout: f32,

    /// List available microphone input devices and exit.
    #[arg(long)]
    list_microphones: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.list_microphones {
        for name in mic::list_input_devices()? {
            println!("{name}");
        }
        return Ok(());
    }
    eprintln!("Loading Parakeet TDT model from {}...", args.model_dir.display());
    let mut transcriber = Transcriber::from_model_dir(&args.model_dir).map_err(|e| {
        format!(
            "{e}\n\nhint: pass --model-dir <DIR> pointing at a directory with \
             encoder-model.int8.onnx, decoder_joint-model.int8.onnx, and vocab.txt \
             (default: {DEFAULT_MODEL_DIR})"
        )
    })?;
    eprintln!("Model loaded.\n");

    let (_mic_stream, rx) = mic::start(Duration::from_secs_f32(args.record_timeout))?;

    let should_stop = Arc::new(AtomicBool::new(false));
    {
        let should_stop = should_stop.clone();
        ctrlc::set_handler(move || should_stop.store(true, Ordering::Relaxed))
            .expect("failed to install Ctrl+C handler");
    }

    let config = streaming::StreamConfig {
        energy_threshold: args.energy_threshold,
        phrase_timeout: Duration::from_secs_f32(args.phrase_timeout),
    };

    let mut last_transcript: Vec<String> = vec![String::new()];
    streaming::run(&mut transcriber, &rx, &config, &should_stop, |lines| {
        last_transcript = lines.to_vec();
        clear_screen();
        for line in lines {
            println!("{line}");
        }
        use std::io::Write;
        let _ = std::io::stdout().flush();
    })?;

    println!("\n\nTranscription:");
    for line in &last_transcript {
        println!("{line}");
    }

    Ok(())
}

fn clear_screen() {
    #[cfg(windows)]
    let _ = std::process::Command::new("cmd").args(["/C", "cls"]).status();
    #[cfg(not(windows))]
    let _ = std::process::Command::new("clear").status();
}
