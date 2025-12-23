## MVP Scope Guardrails (v1.0)

### In scope (must-have)
- Windows desktop app.
- Two windows: Operator (control) + Projector (display).
- Manual verse search and display (works even if AI is off).
- Audio capture + real-time transcription.
- Explicit reference detection (pattern matching).
- Semantic/implicit quote detection (embeddings-based) with top-5 candidates.
- Local Bible database (SQLite + FTS5) with multiple versions and metadata.
- Operator-confirmed display only (no automatic projector updates).
- Keyboard-first interaction for critical flows.
- Session logging + history + export (export format can be minimal in v1, but must exist).
- Offline operation (no internet required during services).

### Out of scope (explicit non-goals for v1)
- Video recording/streaming.
- Presentation slide management (PowerPoint/Keynote).
- Lyrics / worship song display.
- Multi-language support (English only).
- Cloud sync / cloud backup.
- Mobile app.
- Commentary / study notes.
- Automatic display without operator confirmation.

### MVP “must nots” (hard constraints)
- Must not require internet connectivity at runtime.
- Must not auto-display scripture on projector without an operator action.
- Must not omit copyright attribution for restricted translations when used.

