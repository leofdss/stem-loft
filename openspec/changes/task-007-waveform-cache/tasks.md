# Tasks

## 1. Acceptance criteria

- [ ] 1.1 Peaks are computed once (on import, or on first open if no cache exists) and written through the atomic-write path
- [ ] 1.2 The cache is invalidated when its stem is re-imported/replaced
- [ ] 1.3 `waveform_ready` serves peaks from the disk cache when present, without redecoding
- [ ] 1.4 Unit tests cover: peaks computed on first access, the disk cache reused (not recomputed) on the next open, and invalidated after re-import
- [ ] 1.5 `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [ ] 1.6 Reviewed by `review-rust-core`
