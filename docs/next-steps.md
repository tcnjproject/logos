## Next Steps (Build Plan)

This is a practical checklist derived from the PRD roadmap and requirements. It is intentionally explicit about **TBDs** so implementation doesn’t rely on assumptions.

### 0) Confirm “definition of done” for MVP
- **Offline only**: no runtime dependency on internet connectivity.
- **Operator-confirmed display**: never auto-push to projector without operator action.
- **Windows target**: prioritize Windows behavior first (device handling, install, packaging).

### 1) Repo foundation (Week 1–2 in PRD)
- Create baseline Electron app with two windows: Operator + Projector.
- Establish IPC pattern with `contextBridge` security constraints.
- Define initial project structure (main/renderer/shared) and build/package approach.
- Create SQLite database layer for Bible content + metadata (schema + migration approach).

### 2) Data + licensing path (blocker to everything else)
- Decide how Bible text is obtained, stored, and distributed for the 4 default versions (KJV, NIV, AMPC, TPT).
- Decide licensing UX: where to warn, how to attribute, and what is bundled vs user-imported.
- Produce a “bible import pipeline” plan (input format(s), normalization, verification, and indexing).
- **TBD**: confirm whether restricted translations (e.g., NIV/TPT/AMPC) are bundled in v1 installer or must be user-provided.

### 3) Manual mode first (Week 3 in PRD)
- Implement manual search (book/chapter/verse/range) independent of any audio/AI.
- Implement 4-version preview panel and “Push to Screen”.
- Implement projector rendering with formatting defaults and clear-screen behavior.
- Implement keyboard-first interaction for all critical flows (Space/Esc/arrows/Ctrl+F etc.; customizable later if needed).

### 4) Audio + transcription (Weeks 4–5 in PRD)
- Implement audio input selection + level meter + disconnect handling.
- Implement audio calibration wizard outputs (profile per device).
- Integrate streaming/chunked Whisper transcription (model storage strategy is part of offline requirement).
- Render live transcript in Operator window and keep last ~5 minutes visible.
- Add voice activity detection to reduce processing when speech is absent.

### 5) Explicit reference detection (Weeks 6–7 in PRD)
- Implement book name/abbreviation list for all 66 books + ordinals.
- Implement regex parser for explicit formats and verse ranges.
- Emit match cards with confidence tiers, rank explicit above semantic.
- Add context tracking for “verse 5” style follow-ups (**TBD** context window algorithm details).

### 6) Semantic matching (Week 8 in PRD)
- Decide semantic model packaging strategy for offline (Transformers.js model distribution).
- Decide embedding strategy (precompute verse embeddings; prioritize top-100 verses first).
- Implement fast similarity lookup with top-5 results + tiered confidence.
- Establish evaluation harness (offline test corpus of common paraphrases + real sermon samples).

### 7) Display settings + copyright (Week 9 in PRD)
- Implement display settings persistence and live preview.
- Implement auto-scaling for long verses.
- Implement copyright attribution display on projector, per version.

### 8) Logging + polish (Week 10 in PRD)
- Implement session management (date/preacher/title).
- Implement history log and export (format **TBD**).
- Implement “Was this correct?” feedback capture (opt-in telemetry is a later decision).
- Add tutorial overlay + diagnostics export path.

### Immediate decisions (recommended before any coding)
- Packaging/installer target: **MSI vs Squirrel vs other** (**TBD**).
- Whisper.cpp integration method: **native addon vs sidecar process** (**TBD**).
- Bible text licensing strategy for restricted translations (**TBD**, legal required).
- Audio capture library choice for Windows stability (**TBD**).

