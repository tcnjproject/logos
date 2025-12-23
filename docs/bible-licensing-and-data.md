## Bible Licensing and Data (Checklist)

This document exists to prevent accidental “guessing” about Bible text sourcing, packaging, and copyright obligations during implementation.

It is **not** legal advice. It is an engineering checklist to surface decisions and blockers early.

### Scope
- Applies to all Bible text content used by the app (bundled, imported, or updated).
- Applies to projector display (public performance) and any exports (logs/PDF).
- v1 PRD baseline: 4 default versions: **KJV, NIV, AMPC, TPT**.

---

## 1) Decisions Required (Must Answer)

### 1.1 Distribution model (per translation)
For each translation (KJV/NIV/AMPC/TPT), decide one of:
- **Bundled**: shipped inside the installer/app package.
- **User-supplied import**: user provides files during setup.
- **Post-install offline pack**: user installs a local “content pack” from USB/local file (no internet required).

Record the decision per version in `docs/decisions.md`.

### 1.2 Rights + terms verification (per translation)
For each translation, determine:
- Who holds the copyright (if any).
- Whether projector/public display is permitted under the intended use.
- Whether distribution of the text (bundled content) is permitted.
- Whether attribution text is required, and exact wording.
- Whether verse-count/quote-length restrictions exist (and whether they apply to full-text storage).

**TBD**: capture verified terms sources/links (or offline documents) in a secure internal location; do not paste copyrighted license text into the repo unless allowed.

### 1.3 Attribution requirements (per translation)
For each translation, define:
- Required attribution string(s) to show on the projector.
- When to show it (always vs only when that version is displayed).
- Minimum font size / placement constraints if any.
- Whether the operator can hide attribution (default should be “visible” for restricted content).

### 1.4 Compliance UX
Decide how the app will communicate licensing constraints:
- Installer/setup warnings and checkboxes (if any).
- In-app “Bible Versions” settings: license type, requires-license flag, link/help text.
- Whether app can enable restricted translations without the operator confirming they have permission.

---

## 2) Data Sourcing and Import Pipeline

### 2.1 Canonicalization (must define once)
- Canon ordering: 66-book Protestant canon (PRD assumption).
- Book names + abbreviations: canonical list (including ordinals like 1/2/3 John).
- Verse addressing rules (chapter/verse numbering) and how to handle edge cases.

### 2.2 Source formats (must decide)
Supported input formats for Bible text ingestion (choose one or more):
- USX / OSIS / XML
- JSON
- CSV/TSV
- Plain text with structure markers

For each chosen format, document:
- Required fields (book, chapter, verse, text).
- Encoding expectations (UTF-8).
- Handling of headings/section titles/footnotes (include vs strip).

### 2.3 Normalization rules (must specify)
Define how to normalize text before indexing/search:
- Whitespace, punctuation, curly quotes, em-dashes.
- Verse number markers (if present in text).
- Parenthetical notes/footnotes (strip?).
- Case normalization policy for search.

### 2.4 Validation checks (must implement eventually)
For each imported translation:
- Validate counts: expected books present; no missing chapters/verses (or allowlist known differences).
- Validate uniqueness: no duplicate (version, book, chapter, verse).
- Validate safe characters and encoding.
- Validate that verse ranges resolve correctly.

### 2.5 Packaging + updates (offline-friendly)
Define how Bible data is stored and updated without internet:
- One SQLite DB per install vs separate content DBs per version.
- Import-time generation of FTS5 indexes.
- Content pack versioning (schema + data version fields).
- Migration strategy when schema changes.

---

## 3) Database Design Requirements (PRD-driven)

### 3.1 Must support
- Fast lookup by reference (book/chapter/verse) and ranges.
- Full-text search (FTS5) across verses.
- Multi-version retrieval (show 4 versions in parallel).
- Version metadata: license type, attribution text, “requiresLicense”.

### 3.2 Open schema choices (TBD)
Decide one:
- **Shared verse table** keyed by a canonical verse id, with per-version text rows.
- **Per-version verse tables** with a mapping layer.

Document the decision in `docs/decisions.md` and then update `docs/data-models.md` accordingly.

---

## 4) Runtime Behavior Requirements (Compliance-sensitive)

### 4.1 Projector display
- Always show reference + version.
- Show attribution when required by that translation’s terms.
- Ensure attribution updates when operator switches version.

### 4.2 Exports and logs
Decide what exports may contain:
- Reference-only logs (safe default).
- Verse text inclusion in exports (may be restricted per translation).

If verse text export is allowed, define:
- Attribution in exported documents.
- Optional “exclude restricted texts” mode.

---

## 5) Deliverables for “Phase 1: Foundation & Data” (PRD)

To unblock development, produce:
- A verified per-version decision table (distribution model + license type + attribution string + requiresLicense).
- A sample import dataset for at least one version (KJV is a likely candidate) to validate pipeline.
- A repeatable importer spec and test plan (even if the importer is built later).

