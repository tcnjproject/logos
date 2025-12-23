## Open Questions (Must Resolve)

This file captures questions explicitly listed in the PRD plus a few “implementation blockers” implied by the PRD constraints. Add new questions here instead of guessing.

### Product questions (from PRD)
- Multi-operator support (needed in v1 or later?)
- Offline updates (how do users update models/app without internet during service?)
- Bible version imports (supported formats; who supplies text?)
- Presentation integration (PowerPoint/Keynote) — later?
- Telemetry and privacy (opt-in? what is collected?)

### Technical questions (from PRD)
- Embedding model selection (is `all-MiniLM-L6-v2` sufficient offline/accuracy?)
- Database optimization strategy (schema, caching, FTS tuning)
- Audio buffering strategy (chunk sizes, queueing, backpressure)
- Concurrent processing architecture (threads/processes; renderer vs main separation)
- Context window algorithm (how context influences ranking; reset rules)

### UX questions (from PRD)
- Auto-display mode (PRD says no auto-display for v1; is any “assistive auto-select” allowed?)
- Notification sounds (needed for operators?)
- Dark mode (operator window)
- Mobile preview/control (v2? keep out of v1)
- Confidence threshold customization (defaults vs user-set)

### Blockers / must-decide for implementation
- Bible licensing strategy for NIV/AMPC/TPT distribution and projector attribution requirements.
- Offline packaging strategy for Whisper + Transformers.js model files (download at install vs bundled).
- Windows audio capture approach/library selection and device compatibility plan.
- Installer/auto-update mechanism (and how it respects offline constraint).

