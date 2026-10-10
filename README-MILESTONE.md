# Milestone: offline speech-to-text and Bible verse detection

The roadmap is wired into the existing Rust/Iced application. Replaced microphone,
transcript, sample-search, and roadmap code/text is retained in comments. Existing
queue, preview, live-display, layout, and NDI behavior remains in place.

## Run

Install Rust with edition 2024 support, a C/C++ toolchain, and CMake for whisper.cpp.
On macOS, use Xcode Command Line Tools. On Linux, CPAL also requires ALSA development
libraries (for example `libasound2-dev` on Debian/Ubuntu).

From the `logos` project directory, provide these runtime files:

- `models/whisper/ggml-base.en.bin`: a Whisper GGML English model compatible with
  `whisper-rs` 0.16. See the [official whisper.cpp model instructions](https://github.com/ggml-org/whisper.cpp/tree/master/models).
- `data/rhema.db`: a populated database with the schema built by `crates/db`, including
  translations, books, verses, and the FTS5 index.

The existing workspace already has both files. They are not embedded in the binary.
For another installation or working directory, set explicit paths:

```sh
export LOGOS_WHISPER_MODEL="/absolute/path/to/ggml-base.en.bin"
export LOGOS_BIBLE_DB="/absolute/path/to/rhema.db"
cargo run
```

The local provider uses CPU inference, English, and up to four worker threads. It
requires no cloud credentials. Allow microphone access when the operating system
requests it. The NDI runtime retains its existing independent setup requirements.

## Prepare the Bible database

The existing builder reads its inputs from `crates/db/sources` and writes
`crates/db/rhema.db`. Put the Bible source JSON files there, then run:

```sh
cargo run --release --manifest-path crates/db/Cargo.toml
export LOGOS_BIBLE_DB="$PWD/crates/db/rhema.db"
cargo run
```

The builder expects Scrollmapper-style JSON with `books`, `chapters`, and `verses`;
see `crates/db/src/main.rs` for filenames and schema. Missing source files are skipped.
The supplied workspace database contains KJV as its only English translation. KJV
is therefore the new default. NIV, ESV, NLT, and NASB require their own source rows;
selecting an absent translation produces an error, with no substituted text.

## Use

1. Click **Start transcript**. The microphone drives both the existing meter and STT.
2. Speak a reference, for example “John chapter three verse sixteen,”
   “First John three sixteen,” or “Romans eight verses twenty-eight through thirty.”
3. After a speech segment is recognized, its text appears in Live Transcript and
   database verses appear in Recent Detections.
4. Use the existing Present/Queue controls. Detection itself does not present a verse.
   Present retains the app's existing behavior when Go Live is enabled.
5. Stop transcription to close the microphone and cancel its provider. Late events
   from an earlier recording session are ignored.

Book search accepts `John 3:16`, `Romans 8:28-30`, and `Psalm 23`. Context search uses
quoted keyword terms against the selected translation's SQLite FTS5 index. It does
not perform semantic theme/paraphrase matching.

## Architecture and SOLID boundaries

- `src/audio.rs`: device ownership and existing meter capture; forwards PCM through
  a bounded, nonblocking channel.
- `src/speech.rs`: microphone downmix/resampling to 16kHz mono i16, STT session
  ownership, event delivery, and cancellation. The `SttProvider` interface supports
  backend substitution without changing the detection pipeline.
- `crates/stt`: imported Rhema STT providers; local Whisper uses the imported
  `crates/audio` voice activity detector. See [source attribution](crates/RHEMA-SOURCES.md).
- `src/detection.rs`: pure parsing behind the `ReferenceDetector` interface,
  independent of Iced, microphone devices, and SQLite.
- `src/scripture.rs`: `VerseRepository` interface and `SqliteVerseRepository` adapter
  over `rhema-bible`; validates translation availability and complete verse ranges.
- `src/app.rs`: composes these interfaces and coordinates messages. Model loading,
  inference, database startup, searches, and detection lookups run off the GUI thread.
  Request/session IDs reject stale results after query, translation, or recording changes.

This separates responsibilities and gives STT, parsing, and persistence narrow,
replaceable interfaces. Tests substitute a provider and use an isolated database.

## Bounds and limitations

- Whisper emits finalized segments after speech pauses, or roughly ten seconds of
  continuous speech; inference adds latency. This is incremental transcription,
  without word-by-word interim captions.
- Detection supports explicit references across all 66 books, common abbreviations,
  spoken numbers, numbered books, and same-chapter ranges. Database lookup verifies
  that every requested verse exists. Cross-chapter ranges and implicit quote matching
  are outside this milestone.
- The recent transcript window keeps 512 characters for detecting references split
  between speech segments. Displayed transcript text is bounded to about 32KB.
- Recent Detections keeps 30 unique reference/translation pairs. Repeat detections
  move an existing verse to the front. Search results are limited to 30 for Context.
- Audio and event channels are bounded. If inference falls behind, audio frames may
  be dropped to keep the microphone callback responsive. Stopping suppresses late
  results; a running native inference may finish before its worker exits.
- Transcription accuracy depends on microphone quality, background noise, and model.
  Microphone/model/provider errors appear beneath the recording controls; Bible
  errors appear in Search or the transcript status.

## Verify

```sh
cargo check --offline
cargo test --offline
CARGO_TARGET_DIR=target cargo test --offline --manifest-path crates/audio/Cargo.toml
CARGO_TARGET_DIR=target cargo test --offline --manifest-path crates/stt/Cargo.toml --features whisper
# With a local model available:
cargo test --offline local_whisper_model_loads -- --ignored --nocapture
```

Omit `--offline` on a machine that has not downloaded the dependencies yet.
Automated coverage includes audio rate/channel conversion, provider event delivery
and cancellation, reference parsing/deduplication, translation-specific database
lookup, incomplete ranges, missing databases/translations, and chapter/Context search.
These tests do not establish microphone recognition quality or NDI output quality.

Validation in the supplied workspace passed: 10 application tests, 7 imported VAD
tests, and the optional smoke test loading the actual `ggml-base.en.bin` model.
`cargo check --offline`, the STT crate test build, and `git diff --check` also passed.

For a live smoke test, start transcription, speak `John 3:16`, wait for a finalized
segment, verify KJV text in Recent Detections, queue/present it, then stop and restart
recording. Select an absent translation to verify the visible error. Run this on a
machine with microphone access and the runtime model/database files.
