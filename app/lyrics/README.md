# Choir Lyrics Follower

A Rust prototype that listens to a choir through a microphone, figures out where in the song they are, and displays the current lyric line on a projector-friendly web page — advancing automatically as they sing.

Rather than trying to transcribe choral singing from scratch (which is unreliable), it uses **forced alignment**: the full lyrics are known ahead of time, and live speech recognition output is fuzzy-matched against them to track the choir's position.

## How it works

```
microphone (cpal)
      │  mono f32 samples, ring buffer
      ▼
every ~4 seconds: resample to 16 kHz
      │
      ▼
Whisper transcription (whisper-rs / whisper.cpp, fully offline)
      │  imperfect text, e.g. "amazing grace how sweet"
      ▼
fuzzy alignment against known lyric lines
(search window = current line + a few lines ahead)
      │
      ▼
current line index → SSE broadcast → browser display
```

The small lookahead window is what makes this robust: bad transcriptions can't jump the display around the song, and repeated choruses don't confuse it.

## Requirements

- **Rust** (stable) — install via [rustup](https://rustup.rs)
- **C/C++ toolchain** — `whisper-rs` compiles whisper.cpp from source
  - Linux: `sudo apt install build-essential cmake`
  - macOS: `xcode-select --install`
  - Windows: MSVC (Visual Studio Build Tools)
- **A Whisper model file** (one-time download, see below)
- A microphone and a browser for the display

## Setup

### 1. Download a Whisper model

Models are hosted on Hugging Face under `ggerganov/whisper.cpp`. Download one into the project directory, for example:

```sh
curl -L -o ggml-base.en.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin
```

| Model      | Size    | Notes                                    |
|------------|---------|------------------------------------------|
| tiny.en    | ~75 MB  | Fastest, lowest accuracy                 |
| base.en    | ~140 MB | Good starting point                      |
| small.en   | ~465 MB | Noticeably better on difficult audio     |

Start with `base.en`; move up to `small.en` if transcription of the choir is too poor, or down to `tiny.en` if latency is a problem on your hardware.

### 2. Prepare your lyrics

Edit `lyrics.txt`: one **display line** per line of the file, in the order the choir will sing them. Blank lines are ignored. If a verse or chorus repeats, include it again at that point in the file — the aligner moves forward through the file, so repeats must appear where they'll be sung.

### 3. Build and run

```sh
cargo run --release -- ggml-base.en.bin lyrics.txt
```

(Always use `--release` — Whisper in debug mode is far too slow for real time.)

Then open **http://localhost:3000** and make it fullscreen on the projector. Any device on the same network can open `http://<your-ip>:3000` instead, which is handy for a confidence monitor.

The terminal shows what Whisper hears (`heard: ...`) and when the display advances (`>>> line N: ...`). Watch this during rehearsal — it's your main tuning tool.

### GPU acceleration (optional)

In `Cargo.toml`, enable a feature on `whisper-rs`:

```toml
whisper-rs = { version = "0.12", features = ["metal"] }  # macOS
# or
whisper-rs = { version = "0.12", features = ["cuda"] }   # NVIDIA
```

## Tuning

All the knobs live at the top of `src/main.rs`:

| Constant          | Default | Effect                                                                 |
|-------------------|---------|------------------------------------------------------------------------|
| `CHUNK_SECS`      | 4.0     | Audio per transcription pass. Shorter = lower latency, worse accuracy. |
| `LOOKAHEAD`       | 4       | How many lines ahead to search. Larger = recovers faster if lost, but riskier with repeated lyrics. |
| `MATCH_THRESHOLD` | 0.45    | Minimum similarity (0–1) to advance. Raise if the display jumps ahead wrongly; lower if it lags behind. |

**The single biggest factor is mic placement.** A microphone close to the choir, with piano/organ quiet in that mic's mix, will outperform any amount of software tuning. Reverberant rooms and loud accompaniment are the main causes of garbled transcription.

Suggested rehearsal workflow: run the app during practice, watch the `heard:` lines in the terminal, and adjust `MATCH_THRESHOLD` based on the scores you see for correct vs. incorrect matches.

## Project structure

```
choir-lyrics/
├── Cargo.toml
├── lyrics.txt        # your song, one display line per line
└── src/
    └── main.rs       # capture, transcription, alignment, display server
```

Key pieces in `main.rs`:

- `Aligner` — tracks the current line; `feed()` takes a transcript chunk and decides whether to advance
- `token_similarity()` — forgiving bag-of-words + Jaro-Winkler similarity, tolerant of Whisper's mistakes on sung audio
- `start_mic()` / `resample_to_16k()` — audio capture and conversion to Whisper's expected format
- `run_server()` — axum web server pushing the previous/current/next lines to the browser over Server-Sent Events

## Known limitations / roadmap

- **No operator override yet.** Strongly recommended before live use: keyboard controls (arrow keys) to manually jump lines when the choir improvises, skips a verse, or repeats a bridge. No aligner survives spontaneous repeats.
- **Naive resampler.** Linear interpolation is fine for ASR, but the `rubato` crate would be higher quality.
- **Non-overlapping chunks.** Phrases that straddle a chunk boundary can be missed; a sliding window with ~50% overlap would fix this.
- **Forward-only alignment.** The aligner never moves backward. This is intentional (stability), but means a missed advance requires manual correction until override controls exist.
- **English models.** Uses `*.en` models and `language = "en"`. For other languages, use a multilingual model (e.g. `ggml-base.bin`) and change `set_language` in `main.rs`.
- **Prototype status.** This code has not been compiled against your exact crate versions; `whisper-rs` has changed its API between releases, so minor adjustments may be needed. Check the [whisper-rs docs](https://docs.rs/whisper-rs) if the build complains.

## License

Prototype code — use however you like. Note that whisper.cpp and the Whisper models carry their own (MIT) licenses.