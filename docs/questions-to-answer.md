# Questions To Answer (Pre-Build Checklist)

Use this as the single list of **open decisions** to answer before/during implementation. It consolidates:
- `docs/open-questions.md`
- “TBDs (must be decided early)” in `docs/tech-spec-baseline.md`
- the licensing/data decision checklist in `docs/bible-licensing-and-data.md`

Answering these will unblock building without guessing.

---

## A) Product / Scope

- [x] **Multi-operator support**: v1 single operator/single active session; networked multi-operator in v2 (`docs/implementation-decisions.md`).
- [x] **Presentation integration** (PowerPoint/Keynote): later; no v1 integration (`docs/open-questions.md` originally, confirmed “later” in `docs/implementation-decisions.md` scope).
- [x] **Telemetry**: opt-in only, anonymous; never collect transcripts/verses/church data; include payload preview + delete (`docs/implementation-decisions.md`).

## B) Offline Updates & Distribution

- [x] **Offline updates**: offline update packages supported (USB); no forced updates during service; updates only when session stopped (`docs/implementation-decisions.md`).
- [ ] **Installer/updater mechanism**: TBD (MSI vs Squirrel vs other; update channel details) (`docs/next-steps.md:58`).

## C) Bible Text: Licensing, Sourcing, Packaging (Blocker)

### C1) Per-translation distribution model (KJV/NIV/AMPC/TPT)
- [x] For each translation, choose one: **bundled**, **user-supplied import**, or **post-install offline pack**.
  - v1 intent: bundle KJV/NIV/AMPC/TPT only if licensed; user import v1.1+ (`docs/implementation-decisions.md`).

### C2) Rights/terms verification (per translation)
- [ ] Who holds rights (if any), and is projector/public display allowed under intended use? (Partially: user responsible for public display licensing; legal verification still needed) (`docs/implementation-decisions.md`).
- [ ] Is redistribution (bundling the text) allowed? (Partially: “bundle only if licensed”; per-version bundling permissions still TBD) (`docs/implementation-decisions.md`).
- [ ] Are there quote-length / verse-count restrictions that affect local storage and exports? TBD.
- [ ] What is the **exact** required attribution text (wording) and any placement/size rules? (Partially: attribution mandatory + auto-shown; exact strings TBD) (`docs/implementation-decisions.md`).

### C3) Compliance UX (per translation)
- [ ] What warnings/checkboxes appear during setup? TBD.
- [ ] Can restricted translations be enabled without an explicit “I have permission” confirmation? (Partially: app enforces attribution display but does not validate licenses) (`docs/implementation-decisions.md`).
- [ ] How is license status shown in-app (metadata, indicators, links)? TBD.
- [ ] Can the operator hide attribution (default should be “visible” for restricted content)? TBD (current intent: attribution mandatory, auto-shown) (`docs/implementation-decisions.md`).

### C4) Import formats + normalization
- [x] Supported import formats (USX/OSIS/XML, JSON, CSV/TSV, etc.) and required fields.
  - Decision: internal runtime format is normalized JSON/DB schema (not raw OSIS at runtime); user import comes in v1.1+ (`docs/implementation-decisions.md`).
- [ ] Canonicalization rules (book list/abbreviations, ordinals, verse addressing edge cases).
- [ ] Normalization rules for indexing/search (punctuation, quotes, whitespace, footnotes/headings).
- [ ] Validation rules on import (missing verses, duplicates, encoding, known differences allowlist).

### C5) Database schema choice
- [ ] Choose schema approach: **shared canonical verse id + per-version rows** vs **per-version tables + mapping layer**.

### C6) Exports/logging policy (licensing-sensitive)
- [ ] Are exports **reference-only** or can they include verse text?
- [ ] If verse text export is allowed: what attribution is required in exports, and do we need an “exclude restricted texts” mode?

## D) Audio Capture + Transcription (Windows)

- [ ] **Windows audio capture approach/library**: TBD.
- [ ] **Device handling plan**: selection UX, disconnect/reconnect behavior, compatibility/testing matrix.
- [x] **Audio buffering strategy**: ~800ms chunks with 200ms hop; backpressure drops oldest audio when queue > 3 (`docs/implementation-decisions.md`).
- [ ] **Whisper.cpp integration method**: TBD (native addon vs sidecar process).
- [ ] **Whisper model distribution/storage**: partially answered (offline; updated via offline packages); storage/location and bundling vs install-download still TBD.

## E) Semantic Matching / Embeddings

- [x] **Embedding model choice**: `all-MiniLM-L6-v2` (`docs/implementation-decisions.md`).
- [ ] **Transformers.js model distribution/storage**: bundled vs downloaded-at-install vs offline pack; where stored on disk.
- [x] **Embedding strategy**: pre-embed entire Bible at install (`docs/implementation-decisions.md`).

## F) Architecture / Performance

- [ ] **Concurrency architecture**: TBD (threads/processes; what runs in main vs renderer; crash isolation strategy).
- [x] **Context window algorithm**: 12s rolling; reset after >8s silence OR manual search OR new session (`docs/implementation-decisions.md`).
- [ ] **DB optimization approach**: schema tuning, caching strategy, and FTS5 tuning plan to meet targets.

## G) UX Policy Questions

- [x] **Assistive behaviors**: assistive auto-select allowed as highlight only; still requires operator-confirmed push (`docs/implementation-decisions.md`).
- [x] **Notification sounds**: optional, OFF by default (`docs/implementation-decisions.md`).
- [x] **Dark mode**: not in v1 (light mode only); v1.1 (`docs/implementation-decisions.md`).
- [x] **Confidence threshold customization**: fixed thresholds; <75 hidden unless enabled (`docs/implementation-decisions.md`).

---

## Notes

- When you answer an item that affects architecture/licensing, record it in `docs/decisions.md`.
- If new unknowns appear during implementation, add them to this document (and/or `docs/open-questions.md`) instead of guessing.
