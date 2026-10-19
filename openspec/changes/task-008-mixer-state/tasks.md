# Tasks

## 1. Acceptance criteria

- [ ] 1.1 A volume/mute/solo change writes to an atomic/double-buffer the callback reads — never a `Mutex` it can block on
- [ ] 1.2 A mixer change marks the project dirty for the debounced checkpoint (TASK-002), not a write per tick
- [ ] 1.3 Unit tests cover the volume/mute/solo state transitions and the dirty flag being set on a change
- [ ] 1.4 `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [ ] 1.5 Reviewed by `review-rust-core`
