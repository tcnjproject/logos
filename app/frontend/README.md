# Logos

AI Bible assistant

## Features

- **Loading screen** — animated progress bar, "Setting things up"
- **Live Transcript panel** — mic toggle, audio level bars, scrollable transcript
- **Bible Search** — Book search (reference) + Context search (theme/quote) tabs
- **Translation picker** — NIV, KJV, ESV, NLT, NASB dropdown
- **Verse results** — Present or Queue each result
- **Program Preview** — black screen showing the staged verse
- **Live Display** — Go Live toggle (green), broadcasts to output
- **Verse Queue** — add, present, remove queued verses
- **Recent Detections** — AI-detected verses from speech
- **Tour tooltips** — 8-step onboarding overlay (Next/Back/Skip)
- **Update banner** — "Install & Restart" notification
- **Toolbar** — Settings, Broadcast, Display, Captions icons

## Dependency
- [Rust](https://rust-lang.org/)

## Run
- Clone this repo: `git clone https://github.com/fem-ocean/AI-Bible-Assistant-For-Clergy.git`
- `cd app/frontend`
- Run: `cargo run`

## Files

```
src/
  main.rs          # window setup
  app.rs           # state + update (Message enum, Pewbeam struct)
  data.rs          # Verse, Translation, TourStep, LoadingState
  theme.rs         # color constants
  views/
    loading.rs     # splash screen
    main.rs        # 4-panel main layout
    components.rs  # reusable widgets
    waveforms/
       mod.rs
       vumeter.rs
       waveform.rs
```

## TODO:
## Integration with Parakeet STT

To wire up the Python WebSocket backend from earlier:
- Add `tokio-tungstenite` to Cargo.toml
- Add a `Subscription` in `app.rs` that connects to `ws://127.0.0.1:8765/transcribe`
- Map incoming transcripts to `Message::TranscriptUpdated`
- Parse detected verse references and push to `Message::AddToDetections`