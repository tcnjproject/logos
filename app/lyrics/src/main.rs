//! Choir lyric follower — prototype
//!
//! Pipeline:
//!   mic (cpal) -> ring buffer -> every CHUNK_SECS: downmix+resample to 16kHz
//!   -> whisper-rs transcribe -> fuzzy match against known lyric lines
//!   -> advance current line -> broadcast to browser via SSE
//!
//! Usage:
//!   1. Download a Whisper model, e.g.:
//!      https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin
//!   2. Put your lyrics in lyrics.txt, one display line per line.
//!   3. cargo run --release -- ggml-base.en.bin lyrics.txt
//!   4. Open http://localhost:3000 fullscreen on the projector.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use tokio::sync::watch;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// Seconds of audio per transcription chunk. 3–5s is a good trade-off:
/// shorter = lower latency, longer = better transcription accuracy.
const CHUNK_SECS: f32 = 4.0;
/// Whisper expects 16 kHz mono f32.
const WHISPER_RATE: u32 = 16_000;
/// How many lines ahead of the current position we search for a match.
/// Small window = robust to repeated choruses and bad transcription.
const LOOKAHEAD: usize = 4;
/// Minimum normalized similarity (0..1) required to advance the display.
const MATCH_THRESHOLD: f64 = 0.45;

// ---------------------------------------------------------------------------
// Lyric alignment
// ---------------------------------------------------------------------------

struct Aligner {
    /// Normalized lyric lines (lowercase, punctuation stripped).
    lines_norm: Vec<String>,
    /// Original lines for display.
    lines: Vec<String>,
    current: usize,
}

fn normalize(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

impl Aligner {
    fn new(lyrics: &str) -> Self {
        let lines: Vec<String> = lyrics
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect();
        let lines_norm = lines.iter().map(|l| normalize(l)).collect();
        Self { lines_norm, lines, current: 0 }
    }

    /// Feed a chunk of transcribed text. Returns Some(new_index) if we
    /// should advance the display.
    fn feed(&mut self, transcript: &str) -> Option<usize> {
        let heard = normalize(transcript);
        if heard.split_whitespace().count() < 2 {
            return None; // too little signal, ignore
        }

        let start = self.current;
        let end = (self.current + LOOKAHEAD).min(self.lines_norm.len().saturating_sub(1));

        let mut best: Option<(usize, f64)> = None;
        for i in start..=end {
            // Compare against this line, and this line joined with the next
            // (Whisper chunks often straddle two lyric lines).
            let mut candidates = vec![self.lines_norm[i].clone()];
            if i + 1 < self.lines_norm.len() {
                candidates.push(format!("{} {}", self.lines_norm[i], self.lines_norm[i + 1]));
            }
            let score = candidates
                .iter()
                .map(|c| token_similarity(&heard, c))
                .fold(0.0_f64, f64::max);

            if best.map_or(true, |(_, s)| score > s) {
                best = Some((i, score));
            }
        }

        match best {
            Some((idx, score)) if score >= MATCH_THRESHOLD && idx >= self.current => {
                self.current = idx;
                Some(idx)
            }
            _ => None,
        }
    }
}

/// Order-insensitive token overlap blended with Jaro-Winkler on the joined
/// strings. Choral transcription mangles word order and drops words, so a
/// bag-of-words overlap is more forgiving than pure edit distance.
fn token_similarity(heard: &str, expected: &str) -> f64 {
    let h: Vec<&str> = heard.split_whitespace().collect();
    let e: Vec<&str> = expected.split_whitespace().collect();
    if h.is_empty() || e.is_empty() {
        return 0.0;
    }
    let mut hits = 0usize;
    for w in &e {
        if h.iter().any(|hw| strsim::jaro_winkler(hw, w) > 0.85) {
            hits += 1;
        }
    }
    let overlap = hits as f64 / e.len() as f64;
    let jw = strsim::jaro_winkler(heard, expected);
    0.7 * overlap + 0.3 * jw
}

// ---------------------------------------------------------------------------
// Audio capture
// ---------------------------------------------------------------------------

/// Starts the default input device and pushes mono f32 samples (at the
/// device's native rate) into a shared buffer. Returns (stream, native_rate).
fn start_mic(buffer: Arc<Mutex<Vec<f32>>>) -> Result<(cpal::Stream, u32)> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .context("no input device found")?;
    let config = device.default_input_config()?;
    let channels = config.channels() as usize;
    let rate = config.sample_rate().0;
    eprintln!(
        "Mic: {} | {} ch @ {} Hz",
        device.name().unwrap_or_default(),
        channels,
        rate
    );

    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _| {
            let mut buf = buffer.lock().unwrap();
            // Downmix to mono.
            for frame in data.chunks(channels) {
                let s: f32 = frame.iter().sum::<f32>() / channels as f32;
                buf.push(s);
            }
        },
        |err| eprintln!("audio error: {err}"),
        None,
    )?;
    stream.play()?;
    Ok((stream, rate))
}

/// Naive linear resampler to 16 kHz. Fine for speech recognition; swap in
/// the `rubato` crate if you want higher quality.
fn resample_to_16k(input: &[f32], from_rate: u32) -> Vec<f32> {
    if from_rate == WHISPER_RATE {
        return input.to_vec();
    }
    let ratio = from_rate as f64 / WHISPER_RATE as f64;
    let out_len = (input.len() as f64 / ratio) as usize;
    (0..out_len)
        .map(|i| {
            let pos = i as f64 * ratio;
            let idx = pos as usize;
            let frac = (pos - idx as f64) as f32;
            let a = input[idx.min(input.len() - 1)];
            let b = input[(idx + 1).min(input.len() - 1)];
            a + (b - a) * frac
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Web display (SSE)
// ---------------------------------------------------------------------------

const PAGE: &str = r#"<!doctype html><html><head><meta charset="utf-8">
<style>
  body { background:#000; color:#fff; font-family:Georgia,serif;
         display:flex; flex-direction:column; justify-content:center;
         align-items:center; height:100vh; margin:0; text-align:center; }
  #prev, #next { opacity:.35; font-size:4vw; }
  #line { font-size:7vw; margin:2vh 0; }
</style></head><body>
<div id="prev"></div><div id="line">Waiting for choir…</div><div id="next"></div>
<script>
  const es = new EventSource('/events');
  es.onmessage = (e) => {
    const d = JSON.parse(e.data);
    document.getElementById('prev').textContent = d.prev || '';
    document.getElementById('line').textContent = d.line || '';
    document.getElementById('next').textContent = d.next || '';
  };
</script></body></html>"#;

async fn run_server(rx: watch::Receiver<String>) {
    use axum::response::sse::{Event, Sse};
    use axum::{routing::get, Router};
    use futures::stream::StreamExt;
    use tokio_stream::wrappers::WatchStream;

    let app = Router::new()
        .route("/", get(|| async { axum::response::Html(PAGE) }))
        .route(
            "/events",
            get(move || {
                let rx = rx.clone();
                async move {
                    let stream = WatchStream::new(rx)
                        .map(|json| Ok::<_, std::convert::Infallible>(Event::default().data(json)));
                    Sse::new(stream)
                }
            }),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    eprintln!("Display: http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let model_path = args.next().context("usage: lyrics <model.bin> <lyrics.txt>")?;
    let lyrics_path = args.next().context("usage: lyrics <model.bin> <lyrics.txt>")?;

    let lyrics = std::fs::read_to_string(&lyrics_path)?;
    let mut aligner = Aligner::new(&lyrics);
    eprintln!("Loaded {} lyric lines", aligner.lines.len());

    // Whisper
    let ctx = WhisperContext::new_with_params(&model_path, WhisperContextParameters::default())
        .context("failed to load whisper model")?;
    let mut state = ctx.create_state()?;

    // Mic
    let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
    let (_stream, native_rate) = start_mic(buffer.clone())?;
    let chunk_len = (native_rate as f32 * CHUNK_SECS) as usize;

    // Display server on a background tokio runtime.
    let (tx, rx) = watch::channel(String::from("{}"));
    std::thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(run_server(rx));
    });

    let push_display = |aligner: &Aligner, tx: &watch::Sender<String>| {
        let i = aligner.current;
        let get = |j: isize| -> String {
            if j < 0 || j as usize >= aligner.lines.len() {
                String::new()
            } else {
                aligner.lines[j as usize].clone()
            }
        };
        let json = format!(
            r#"{{"prev":{:?},"line":{:?},"next":{:?}}}"#,
            get(i as isize - 1),
            get(i as isize),
            get(i as isize + 1)
        );
        let _ = tx.send(json);
    };
    push_display(&aligner, &tx);

    eprintln!("Listening… (Ctrl-C to quit)");
    loop {
        std::thread::sleep(Duration::from_millis(250));

        // Take a chunk when enough audio has accumulated.
        let chunk: Option<Vec<f32>> = {
            let mut buf = buffer.lock().unwrap();
            if buf.len() >= chunk_len {
                Some(buf.drain(..chunk_len).collect())
            } else {
                None
            }
        };
        let Some(chunk) = chunk else { continue };

        let audio16k = resample_to_16k(&chunk, native_rate);

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("en"));
        params.set_print_progress(false);
        params.set_print_special(false);
        params.set_print_realtime(false);
        params.set_no_timestamps(true);
        params.set_suppress_blank(true);

        if state.full(params, &audio16k).is_err() {
            continue;
        }

        let mut transcript = String::new();
        let n = state.full_n_segments();
        for s in 0..n {
            if let Some(segment) = state.get_segment(s) {
                if let Ok(text) = segment.to_str() {
                    transcript.push_str(text);
                    transcript.push(' ');
                }
            }
        }
        let transcript = transcript.trim();
        if transcript.is_empty() {
            continue;
        }
        eprintln!("heard: {transcript}");

        if let Some(new_idx) = aligner.feed(transcript) {
            eprintln!(">>> line {}: {}", new_idx, aligner.lines[new_idx]);
            push_display(&aligner, &tx);
        }
    }
}
