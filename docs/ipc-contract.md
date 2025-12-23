## IPC Contract (Electron)

This lists IPC channels described in the PRD. Payload schemas are intentionally marked **TBD** until the data model is finalized during implementation.

### Main → Renderer
- `audio-status` — audio device connection status + quality (**TBD payload**)
- `audio-quality-update` — real-time audio quality metrics (**TBD payload**)
- `stt-transcript` — live transcript updates (**TBD payload**)
- `reference-detected` — detected reference + confidence tier (**TBD payload**)
- `verse-loaded` — verse text + copyright attribution (**TBD payload**)
- `projector-connected` — projector display status (**TBD payload**)
- `heartbeat` — IPC health check (every 5s) (**TBD payload**)

### Renderer → Main
- `get-audio-devices` — request available audio devices (**TBD response**)
- `set-audio-device` — set active audio device (**TBD payload**)
- `run-audio-calibration` — start calibration wizard (**TBD payload**)
- `start-listening` — start audio capture (**TBD payload**)
- `stop-listening` — stop audio capture (**TBD payload**)
- `search-verse` — manual verse search (**TBD payload/response**)
- `display-verse` — push verse to projector (**TBD payload**)
- `navigate-verse` — previous/next verse (**TBD payload**)
- `update-settings` — save display settings (**TBD payload**)
- `update-shortcuts` — save keyboard shortcuts (**TBD payload**)
- `export-log` — export session log (**TBD payload/response**)
- `log-operator-feedback` — log “was this correct?” response (**TBD payload**)
- `request-diagnostics` — get system diagnostic data (**TBD response**)
- `heartbeat-ack` — acknowledge heartbeat (**TBD payload**)

### Notes / constraints
- Operator-confirmed display: `display-verse` is only invoked by explicit operator action.
- Offline requirement: no IPC action should assume network availability.

