## Definition of Done (v1.0 MVP)

This is the minimum bar for calling v1 “done”. It is derived from `sermon_assistant_prd_v2.md` and is intended to be measurable.

### 1) Core user flows work end-to-end
- Operator can complete a full service workflow: start session → see transcript → see matches → preview → push to projector → clear screen → end session.
- Manual mode works independently of audio/AI: select book/chapter/verse/range → preview → push to projector.
- Two-window behavior works reliably: Operator window controls Projector window via IPC; projector can be full-screen on a second display.

### 2) Hard constraints are met
- Offline operation: app functions during services with no internet connectivity.
- Operator-confirmed display: no automatic projector updates without explicit operator action.
- English-only scope (no multi-language in v1).

### 3) Performance targets are met (on defined test hardware)
- End-to-end: scripture visible on projector within 3–4 seconds of verbal reference on recommended hardware (and 4–5 seconds on minimum hardware), measured from spoken reference to display.
- STT updates appear with <2 seconds latency (target) with stable streaming behavior.
- Semantic candidate retrieval completes within ~500ms target (where enabled).
- DB performance meets PRD targets: reference lookups <50ms; full-text searches <200ms (representative dataset).

### 4) Detection quality meets MVP thresholds
- Explicit reference detection ≥95% accuracy on a defined test set of explicit references and common variants/abbreviations.
- Semantic detection meets PRD targets (top-100 most-quoted verses ≥75% accuracy; broader set ≥60%) on a defined paraphrase test set.
- False positives remain below the PRD threshold (<5%) on evaluation corpus.

### 5) Display quality meets operator needs
- Projector display supports readable typography and auto-scaling for long passages.
- Reference + translation/version label is always visible.
- Clear-screen behavior is instant and reliable (e.g., `Esc`).

### 6) Licensing/copyright compliance gates are satisfied
- Per-translation distribution model is decided (bundled vs user-imported vs offline content pack) and recorded in `docs/decisions.md`.
- Each translation has metadata: license type, requiresLicense flag, attribution string(s).
- When a restricted translation is displayed, required attribution appears on the projector.
- Export/log behavior is compliant with chosen licensing policy (reference-only default unless explicitly permitted).

### 7) Reliability and recovery
- App runs through a 2-hour service without crashes on test hardware (target: 99%+ session uptime as per PRD intent).
- Audio device disconnect is detected and surfaced; app continues functioning in manual mode.
- Errors are handled without breaking the UI (no unhandled exceptions visible to operator).
- Basic diagnostics export exists (contents TBD but must help debug audio/model/db issues offline).

### 8) Packaging and installability (Windows)
- A repeatable Windows installer/build artifact exists and can be installed/uninstalled cleanly.
- Model/data storage locations are defined and work offline (Whisper model + semantic model + Bible DB).
- First-run setup path works (including any licensing warnings/steps).

### 9) Documentation is sufficient for building and operating
- `docs/` is up to date (scope, IPC list, data models, open questions, licensing/data checklist).
- A minimal operator help path exists (in-app tutorial or equivalent quickstart) aligned to PRD expectations.

### Blockers (must be resolved before claiming “done”)
- Bible text sourcing/licensing decisions for NIV/AMPC/TPT (and any bundling vs import approach).
- Windows packaging strategy (installer/updater approach) and offline model distribution strategy.

