## Docs (Source-of-Truth Pack)

This folder exists to help builders (humans + agents) implement the **AI Sermon Assistant** with minimal guesswork.

### Source of truth
- Primary: `sermon_assistant_prd_v2.md` (Approved PRD).
- If anything conflicts, default to the PRD and record proposed changes in `docs/decisions.md`.

### Rules to prevent hallucination
- Do not invent features, integrations, or UI that are not explicitly in the PRD.
- When something is unclear, mark it **TBD** and add it to `docs/open-questions.md`.
- Keep v1 aligned to “offline”, “operator-confirmed display”, and “Windows desktop app”.

### Documents
- `docs/next-steps.md`: concrete execution plan and immediate tasks.
- `docs/mvp-scope.md`: in-scope/out-of-scope guardrails for v1.
- `docs/tech-spec-baseline.md`: baseline tech stack, constraints, and performance targets.
- `docs/ipc-contract.md`: Electron IPC channel list (payloads TBD).
- `docs/data-models.md`: required data structures (from PRD; details TBD where not specified).
- `docs/definition-of-done.md`: measurable release criteria for v1.
- `docs/open-questions.md`: decisions and unknowns that must be resolved.
- `docs/decisions.md`: lightweight decision log (what/why/date/owner).
- Product backlog: [Link to Product Backlog on google sheet](https://docs.google.com/spreadsheets/d/1Dbot-Dtra6cXMyz8mLQyLjB2X4M_St0KuiwVgjuU7XU/edit?usp=sharing) (TBD)
- Team Charter: [Link to Team charter](https://docs.google.com/document/d/1hFnezNtSZtJnvObAf-bmTZHjYu_d9SEv5sMrDemCtY8/edit?tab=t.0#heading=h.r036a3oby8l5) (TBD)
