# Logos

An AI-powered desktop application for automatically detecting and presenting Bible verses and lyrics during a live service.


## Current state

The speech-to-text and verse-detection milestone is implemented. See
[README-MILESTONE.md](README-MILESTONE.md) for setup, architecture, validation, and limits.

- **Live transcript** — one microphone stream supplies the waveform/VU meter and local
  `rhema-stt` Whisper inference. Final speech segments update the transcript through an
  Iced subscription. Transcription runs offline without an API key.
- **Verse detection** — numeric and spoken references are parsed from recent transcript
  context, loaded through `rhema-bible`, and added to Recent Detections. Use the existing
  Present/Queue controls to select what goes on air.
- **Bible search** — reference, chapter, range, and Context keyword searches use SQLite.
  The translation picker loads the actual selected translation and reports unavailable
  translations. KJV is the initial selection.
- **Queue / Preview / Live / NDI**, onboarding, and layout retain their existing behavior.
- **Status messages** show microphone/model/provider failures and Bible lookup errors.

<!-- Previous current-state documentation retained for reference:
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

-->

## Features

- Live Transcript panel — mic toggle, real-time waveform/VU meter, scrollable transcript
- Bible Search — Book search (reference) and Context search (theme/quote) tabs
- Translation picker — NIV, KJV, ESV, NLT, NASB
- Program Preview — staged verse shown before going live
- Live Display — Go Live toggle, broadcasts the staged verse over NDI
- Verse Queue — add, present, remove queued verses
<!-- - Recent Detections panel (UI ready, not yet populated by real detections) -->
- Recent Detections panel populated by transcript reference detection
- 8-step onboarding tour overlay (Next/Back/Skip)
- Resizable/draggable 4-pane layout (`iced::widget::pane_grid`)

## Prerequisites

- [Rust](https://rust-lang.org/) (edition 2024 — install via [rustup](https://rustup.rs/))

## Build & run

```sh
git clone https://github.com/tcnjproject/logos.git
cd logos
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
  assets/              # icons (mic, start/stop, settings, help) and logo images
  crates/                 # Third party crates
    bible/             # [rhema-bible](https://github.com/openbezal/rhema/tree/main/src-tauri/crates/bible)
    db/                # Ported from https://github.com/openbezal/rhema/blob/main/data/build-bible-db.ts
    broadcast/         # [rhema-broadcast](https://github.com/openbezal/rhema/tree/main/src-tauri/crates/broadcast)
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

## Implemented roadmap

- Local `rhema-stt` Whisper provider and cancellation lifecycle.
- Iced STT subscription feeding `Message::TranscriptUpdated`.
- Reference parsing and asynchronous `Message::AddToDetections` updates.
- Translation-specific verse loading and search through `rhema-bible`.

See [milestone setup and verification](README-MILESTONE.md). Model and database files
are runtime inputs and are not embedded in the executable.

<!-- Previous roadmap retained for reference:
## Roadmap

Wiring up real speech-to-text and verse detection:

- Use [`rhema-stt`](https://github.com/openbezal/rhema/tree/main/src-tauri/crates/stt)
- Add a `Subscription` that connects to the STT and maps incoming transcripts to
  `Message::TranscriptUpdated`
- Parse detected verse references from the transcript and push them via
  `Message::AddToDetections`
- Load verses from `rhema-bible`

-->

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option. The NDI
runtime bundled under `crates/broadcast/ndi/` is separate proprietary software owned by Vizrt NDI
AB (NewTek) — see [LICENSE](LICENSE) for details.
