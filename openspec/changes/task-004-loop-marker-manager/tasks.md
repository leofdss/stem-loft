# Tasks

## 1. Acceptance criteria

- [ ] 1.1 `endSec` beyond the project's duration (longest stem's `durationSec`) is rejected, not silently clamped
- [ ] 1.2 Current markers are exposed to `Audio Engine` for loop playback
- [ ] 1.3 Unit tests cover: a marker within duration is accepted, `endSec` beyond duration is rejected
- [ ] 1.4 `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [ ] 1.5 Reviewed by `review-rust-core`
