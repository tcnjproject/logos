# Rhema source attribution

`stt/src` was imported from [OpenBezal Rhema's STT crate](https://github.com/openbezal/rhema/tree/main/src-tauri/crates/stt).
`audio/src/{types,meter,vad}.rs` was imported from its [audio crate](https://github.com/openbezal/rhema/tree/main/src-tauri/crates/audio).
Retrieved October 8, 2026. The upstream Whisper file's Git blob is
`01700bac50dd74388b7875fb7c5416f51bcad360`; the upstream VAD file's blob is
`1987e74704b1f83258a42a7382a2dd6e7a91f9c8`.

The upstream MIT license is retained in [LICENSE-RHEMA](LICENSE-RHEMA).
Manifests use explicit dependencies because Logos does not inherit Rhema's workspace.
The audio crate includes only the VAD modules required by Whisper; Logos owns microphone capture.
Whisper uses its portable CPU backend here. Cloud providers remain available in the library,
but the Logos application selects local Whisper and requires no API key.

Local adaptations: REST exports follow their feature flag, and Whisper emits `Connected`
after successful model loading. Superseded lines are retained as comments.
