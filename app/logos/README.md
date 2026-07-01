# Logos

Desktop control-room app for presenting Bible verses during a live service.
Built with [Rust](https://rust-lang.org/) and [iced](https://github.com/iced-rs/iced), with an
NDI output pipeline (`rhema-broadcast`) for sending the live verse to broadcast/streaming software.

## Current state

- **Live transcript panel** — captures real microphone audio (`cpal`) and drives the waveform/VU
  meter. There is no speech-to-text or automatic verse detection wired in yet, so "Recent
  Detections" stays empty (see [Roadmap](#roadmap)).
- **Bible search** — Book/Context tabs and translation picker are functional, but results come
  from a small hardcoded sample set in `app.rs` (`sample_search_results`), not from `bible.json`.
- **Queue / Preview / Live / Go Live / NDI output** — fully wired. Presenting or going live pushes
  the verse to a background worker thread that renders it and sends it out over NDI.
- **Onboarding tour and update banner** — implemented with local/mock state (no real update
  server check).

## Features

- Loading screen with animated progress bar
- Live Transcript panel — mic toggle, real-time waveform/VU meter, scrollable transcript
- Bible Search — Book search (reference) and Context search (theme/quote) tabs
- Translation picker — NIV, KJV, ESV, NLT, NASB
- Verse results — Present or Queue each result
- Program Preview — staged verse shown before going live
- Live Display — Go Live toggle, broadcasts the staged verse over NDI
- Verse Queue — add, present, remove queued verses
- Recent Detections panel (UI ready, not yet populated by real detections)
- 8-step onboarding tour overlay (Next/Back/Skip)
- Update banner ("Install & Restart")
- Resizable/draggable 4-pane layout (`iced::widget::pane_grid`)

## Prerequisites

- [Rust](https://rust-lang.org/) (edition 2024 — install via [rustup](https://rustup.rs/))
- A working audio input device for the Live Transcript panel
- For NDI output: the runtime library is bundled per-platform under
  `crates/broadcast/ndi/` (`Processing.NDI.Lib.x64.dll` on Windows, `libndi.dylib` on macOS,
  `libndi.so` on Linux) and is loaded automatically — no separate NDI SDK install needed to run
  this app. If the library can't be loaded, the app degrades gracefully (NDI badge turns red and
  shows the error) rather than crashing.

## Build & run

```sh
git clone https://github.com/fem-ocean/AI-Bible-Assistant-For-Clergy.git
cd app/logos
cargo run              # debug build
cargo run --release    # optimized build
```

To produce a standalone binary:

```sh
cargo build --release
# binary at target/release/logos(.exe)
```

## Folder structure

```
app/logos/
  Cargo.toml           # package "logos" — iced, tokio, cpal, fontdue, rhema-broadcast
  bible.json           # local scripture data (gitignored; not yet read by the app)
  assets/              # icons (mic, start/stop, settings, help) and logo images
  crates/
    broadcast/         # `rhema-broadcast` crate — NDI FFI bindings + frame types
      src/lib.rs
      src/ndi.rs       # loads the NDI runtime via libloading, start/stop/send_frame
      ndi/             # bundled NDI runtime libraries (dll/dylib/so)
  src/
    main.rs           # iced application entry point, window setup
    app.rs            # Logos state, Message enum, update()/subscription(), pane layout
    audio.rs          # AudioCapture — cpal mic stream, RMS/peak/waveform on a shared buffer
    broadcast.rs      # FrameRenderer — draws verse text into an RGBA frame (fontdue)
    ndi_worker.rs     # background thread owning the NDI session; GUI talks to it via a channel
    data.rs           # Verse, Translation, SearchMode, TourStep, LoadingState
    theme.rs          # color constants
    views/
      mod.rs
      loading.rs      # splash screen
      main.rs         # 4-panel main layout (transcript, search/preview, live, queue/detections)
      components.rs   # reusable widgets (buttons, tour card, update banner, toggle switch)
      waveform/
        mod.rs
        vumeter.rs    # segmented VU meter driven by live mic data
```

## Roadmap

Wiring up real speech-to-text and verse detection (a Python/`onnxruntime` STT service already
exists as a prototype under `app/backend/`):

- Add a WebSocket/HTTP client (e.g. `tokio-tungstenite`) to `app.rs`
- Add a `Subscription` that connects to the STT backend and maps incoming transcripts to
  `Message::TranscriptUpdated`
- Parse detected verse references from the transcript and push them via
  `Message::AddToDetections`
- Load verses from `bible.json` instead of the hardcoded sample set in `sample_search_results`

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option. The NDI
runtime bundled under `crates/broadcast/ndi/` is separate proprietary software owned by Vizrt NDI
AB (NewTek) — see [LICENSE](LICENSE) for details.
