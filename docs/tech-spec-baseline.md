## Technical Baseline (From PRD)

### Product summary
An offline Windows desktop application that listens to live sermon audio, transcribes speech, detects Bible references, and helps an operator quickly confirm and display scripture on a projector.

### Baseline stack (from PRD)
- Desktop framework: Electron 28.x
- UI: React 18.x
- Styling: Tailwind CSS 3.x
- Database: SQLite 3.x (`better-sqlite3`) with FTS5
- Speech-to-text: Whisper.cpp (small.en)
- Semantic search: Transformers.js (all-MiniLM-L6-v2)
- Fuzzy search: Fuse.js

### Architectural shape (from PRD)
- Electron main process owns:
  - audio capture + streaming pipeline
  - Whisper transcription
  - detection engine (explicit + semantic + ranking + context)
  - bible database queries
  - window management and IPC heartbeat
- Two renderer processes:
  - Operator window (transcript + matches + preview + manual tools)
  - Projector window (full-screen verse display + formatting + attribution)

### Performance targets (from PRD)
- End-to-end latency: scripture visible within ~3–4 seconds of verbal reference (recommended hardware); ~4–5s minimum.
- Transcription latency: <2 seconds for STT update (goal).
- Semantic search: ~500ms target.
- DB lookup: <50ms queries; <200ms full-text search.

### Security/privacy baseline
- Offline-first; privacy assumes local processing.
- IPC uses `contextBridge` and avoids insecure renderer access.

### TBDs (must be decided early)
- Windows packaging/installer approach.
- How Whisper model files are distributed and stored offline.
- How Transformers.js model files are distributed and stored offline.
- Audio capture library and device handling strategy on Windows.

