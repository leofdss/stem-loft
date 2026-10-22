# Tasks

## 1. Acceptance criteria

- [x] 1.1 `score_loaded` fires once per parse with `tempo`/`time`/`tuning` from the header
- [x] 1.2 The per-tick real-time payload carries only the active chord/beat, not the whole score (see Real-time visual state (`docs/architecture.md#real-time-visual-state-angular`))
- [x] 1.3 Unit tests cover the chord/beat slice selection for a few `positionSec` values against a small fixed score
- [x] 1.4 `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [x] 1.5 Reviewed by `review-rust-core`
