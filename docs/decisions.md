## Decision Log

Use this file to record decisions that affect scope, architecture, licensing, or user experience.

## Current v1 decisions (consolidated)

Source: `docs/implementation-decisions.md` (treat as the detailed decision pack).

- Date: 2025-12-23
  Owner: edozienwokoye
  Decision: Platform and scope baseline
  Options considered: Windows-only vs cross-platform; online vs offline
  Chosen option: Windows-only (v1), offline-first Electron desktop app; no internet required during service; no auto-display (operator confirmation required)
  Rationale: Align to v1 PRD constraints and reliability during services
  Impact: Architecture and packaging optimized for Windows; two-window model (Operator + Projector) with strict operator control
  Follow-ups: Confirm installer strategy; confirm audio capture library

- Date: 2025-12-23
  Owner: edozienwokoye
  Decision: Operator model roadmap
  Options considered: Multi-operator in v1 vs later
  Chosen option: v1 single operator + single active session; v1.1 local profiles (preferences only); v2 networked/multi-operator
  Rationale: Reduce complexity for MVP
  Impact: No v1 account/profile sync; keep data model simple
  Follow-ups: Define preferences schema for v1.1

- Date: 2025-12-23
  Owner: edozienwokoye
  Decision: Bible versions and licensing posture (v1)
  Options considered: bundle vs user-import vs offline packs per translation
  Chosen option: Bundle KJV/NIV/AMPC/TPT only if licensed; v1.1+ adds user Bible import; runtime uses normalized internal JSON/DB schema (not raw OSIS); projector attribution mandatory and auto-shown per version; app enforces attribution but does not validate licenses
  Rationale: Compliance-aware baseline while keeping v1 install experience straightforward
  Impact: Requires per-version licensing verification before bundling; attribution UI required in projector window
  Follow-ups: Verify per-version redistribution terms; finalize exact attribution strings and setup warnings

- Date: 2025-12-23
  Owner: edozienwokoye
  Decision: STT + embeddings defaults and streaming parameters
  Options considered: Alternative STT/embedding models and chunking strategies
  Chosen option: Whisper offline; embedding model `all-MiniLM-L6-v2` offline; ~800ms chunks with 200ms hop; backpressure drops oldest audio when queue > 3
  Rationale: Meet latency targets with bounded memory/CPU
  Impact: Audio pipeline and worker interfaces sized around these parameters
  Follow-ups: Decide Whisper integration method (native addon vs sidecar); decide model distribution/storage

- Date: 2025-12-23
  Owner: edozienwokoye
  Decision: Detection behavior and confidence policy
  Options considered: Semantic-first vs explicit-first; auto-display vs assistive
  Chosen option: Explicit reference detection primary; semantic secondary; 12s rolling context window; reset after >8s silence OR manual search OR new session; confidence tiers High ≥92, Good 85–91, Possible 75–84, <75 hidden unless enabled; assistive auto-select allowed (highlight only) but push remains operator-confirmed
  Rationale: Predictable operator control with useful assistance
  Impact: Ranking, UI presentation, and shortcuts should reflect these tiers and rules
  Follow-ups: Define “silence” detection method and thresholds

- Date: 2025-12-23
  Owner: edozienwokoye
  Decision: UX and telemetry guardrails
  Options considered: Dark mode in v1; notification defaults; telemetry breadth
  Chosen option: Operator window light mode only (v1); dark mode v1.1; notification sounds optional and OFF by default; telemetry opt-in only, anonymous, with payload preview + delete; never collect transcripts, verses, or church data
  Rationale: Keep v1 simple and privacy-safe
  Impact: Telemetry implementation must aggressively minimize/aggregate and provide review/delete UX
  Follow-ups: Define exact telemetry event list (if any)

- Date: 2025-12-23
  Owner: edozienwokoye
  Decision: Update policy
  Options considered: Auto-update vs manual/offline packages
  Chosen option: No forced updates during service; offline update packages supported (USB); updates only applied when session stopped
  Rationale: Avoid service disruption; respect offline constraint
  Impact: Updater/installer must support offline artifacts and safe apply timing
  Follow-ups: Choose installer/updater mechanism (MSI vs Squirrel vs other)

### Template
- Date:
- Owner:
- Decision:
- Options considered:
- Chosen option:
- Rationale:
- Impact:
- Follow-ups:
