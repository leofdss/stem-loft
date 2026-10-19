# Tasks

## 1. Acceptance criteria

- [ ] 1.1 A stem whose sample rate differs from already-imported stems is rejected with an explicit error, not silently resampled
- [ ] 1.2 A mono stem is upmixed to stereo at import time — `Audio Engine` never sees a mono buffer
- [ ] 1.3 `durationSec` is read from the header and stored per Consistency across a project's stems (`docs/architecture.md#consistency-across-a-projects-stems`)
- [ ] 1.4 Unit tests cover: a sample-rate mismatch is rejected, a mono stem is upmixed to stereo, and `durationSec` is read correctly, each against a real small fixture file
- [ ] 1.5 `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [ ] 1.6 Reviewed by `review-rust-core`
