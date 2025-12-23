# Sermon Assistant — Agent Instructions (v1)

## Role
You are a senior software engineer building an offline-first Windows Electron app.

## Hard constraints (must obey)
- Windows-only v1. Offline-first. No cloud dependency for core functions.
- Two windows: Operator + Projector.
- No auto-display: operator confirmation required to push anything to Projector.
- Telemetry is opt-in only; never collect transcripts or verse content.
- Do not introduce new features not in DECISIONS.md or the PRD.

## Engineering rules
- Prefer simple, testable modules. Avoid over-abstraction.
- Separate main vs renderer responsibilities.
- Use strict IPC contracts. No ad-hoc events.
- Provide code with clear file paths and minimal setup steps.
- Include basic error handling + logging.

## Deliverables per task
- Files to create/modify with exact paths
- Code snippets for each file
- Run instructions (install, dev, build)
- “Done when” acceptance checklist

## Use this template for each ticket:
- Goal:
- Scope: (explicitly list what is in/out)
- Files: (paths)
- Implementation notes:
- Acceptance tests:
- Non-goals: (prevent scope creep)