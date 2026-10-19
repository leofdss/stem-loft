# Tasks

## 1. Acceptance criteria

- [ ] 1.1 Decoding and any I/O happen outside the `cpal` callback; the callback only reads already-prepared buffers
- [ ] 1.2 No allocation, lock, or I/O inside the callback (checked against the `realtime-audio-safety` checklist)
- [ ] 1.3 Unit tests cover the decode → ring-buffer handoff in isolation, without needing a real `cpal` device
- [ ] 1.4 A human confirms audible, glitch-free playback on at least one real device
- [ ] 1.5 `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [ ] 1.6 Reviewed by `review-rust-core`
