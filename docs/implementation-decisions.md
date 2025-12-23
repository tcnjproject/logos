# IMPLEMENTATION DECISIONS & AGENT PACK

This document is the single source of truth for v1 implementation. Agents must not guess beyond what is written here.

## 1. DECISIONS.md (Hard Decisions – v1)

### Platform & Scope

- **OS**: Windows only (v1)

- **App Type**: Offline‑first Electron desktop app

- **Internet**: Not required during service

- **Auto‑display**: ❌ Not allowed (operator confirmation required)

### Operators

- **v1**: Single operator, single active session

- **v1.1**: Multiple local operator profiles (preferences only)

- **v2**: Networked / multi‑operator

### Bible Versions

- **Bundled in v1**: KJV, NIV, AMPC, TPT (only if licensed)

- **User Bible import**: v1.1+

- **Internal format**: Normalized JSON/DB schema (not raw OSIS at runtime)

- **Attribution**: Mandatory, auto‑shown per version

### Audio & AI

- **Speech‑to‑Text**: Whisper (offline)

- **Embedding model**: all‑MiniLM‑L6‑v2 (offline)

- **Chunking**: ~800ms chunks, 200ms hop

- **Backpressure**: Drop oldest audio when queue > 3

### Detection Logic

- **Modes**:

    - Explicit reference detection (primary)

    - Semantic matching (secondary)

- **Context window**: 12 seconds rolling

- **Context reset**: >8s silence OR manual search OR new session

### Confidence Handling

- High ≥ 92

- Good 85–91

- Possible 75–84

- <75 hidden unless enabled

- Assistive auto‑select allowed (highlight only)

### UX

- **Operator window**: Light mode only (v1)

- **Dark mode**: v1.1

- **Notification sounds**: Optional, OFF by default

- **Mobile control**: v2

### Telemetry

- **Opt‑in only**

- **Anonymous**

- No transcripts, no verses, no church data

- User‑visible payload preview + delete option

### Updates

- No forced updates during service

- Offline update packages supported (USB)

- Updates only applied when session stopped

## 2. ASSUMPTIONS.md (Explicit Assumptions)

These assumptions unblock development and may be revisited post‑v1.

### Licensing

- User is responsible for public display licensing where required

- App enforces attribution display but not license validation

### Performance

- Typical church hardware: mid‑range Windows PC, USB audio interface

- Embedding accuracy acceptable at MiniLM scale

### Usage Pattern

- One sermon at a time

- One projector output

- Operator actively watching suggestions

### Data Volume

- Entire Bible pre‑embedded at install

- DB size acceptable for local SSD storage

### Security

- No cloud sync

- No authentication beyond local OS user

## 3. AGENT TASK BREAKDOWN (Non‑Hallucinating Plan)

### PHASE 0 – Foundations

**Goal**: App boots, windows render, IPC works

Tasks:

- Create Electron app shell (main + renderer)

- Implement window manager:

    - Operator window

    - Projector window (borderless, black bg)

- IPC contract for:

    - Verse selection

    - Session state

Acceptance:

- Two windows launch reliably

- Projector mirrors selected verse text

### PHASE 1 – Audio Pipeline

**Goal**: Reliable offline audio capture → PCM stream

Tasks:

- Enumerate Windows audio input devices

- Select active device

- Stream PCM → chunk buffer

- Implement backpressure rules

Acceptance:

- Live waveform debug view

- Stable stream for 60+ minutes

### PHASE 2 – Speech‑to‑Text Worker

**Goal**: Offline transcription with acceptable latency

Tasks:

- Integrate Whisper model (worker thread)

- Handle chunk queue

- Emit partial + final transcript segments

Acceptance:

- <2s latency

- No UI blocking

### PHASE 3 – Bible Data Layer

**Goal**: Fast verse lookup + semantic search

Tasks:

- SQLite schema creation

- FTS5 index for text search

- Embedding table

- Pre‑embedding pipeline

Acceptance:

- Explicit lookup <50ms

- Semantic query <200ms

### PHASE 4 – Detection Engine

**Goal**: Correct verse suggestions

Tasks:

- Explicit reference parser

- Semantic similarity ranking

- Context window scoring

- Confidence scoring

Acceptance:

- Correct verse in top 3 for scripted test sermons

### PHASE 5 – Operator UX

**Goal**: Zero‑surprise live usage

Tasks:

- Live transcript panel

- Ranked suggestion list

- Confidence indicators

- Keyboard shortcuts (confirm / reject)

Acceptance:

- Operator can run full mock service hands‑free

### PHASE 6 – Projector Output

**Goal**: Clean, readable scripture display

Tasks:

- Verse formatting

- Attribution footer

- Font scaling

- Fade transitions

Acceptance:

- Legible at distance

- Attribution always visible

### PHASE 7 – Installer & Updates

**Goal**: Church‑proof deployment

Tasks:

- Offline installer (models included)

- Optional web installer

- Update import workflow

- Versioned model folders

Acceptance:

- Fresh install works without internet

### PHASE 8 – Telemetry & Privacy

**Goal**: Trust by default

Tasks:

- Opt‑in dialog

- Payload preview UI

- Local data delete

Acceptance:

- Zero telemetry without consent

### FINAL RULE FOR AGENTS

If it’s not written here or in the PRD, do not invent it. Ask before implementing.