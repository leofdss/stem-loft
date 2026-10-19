# Tasks

## 1. Acceptance criteria

- [ ] 1.1 The start-of-loop buffer is prepared before the callback reaches `endSec`
- [ ] 1.2 The crossfade is pure math over already-decoded samples inside the callback — no new allocation/I/O
- [ ] 1.3 Unit tests cover the crossfade math (fade curve) over synthetic sample buffers, independent of `cpal`
- [ ] 1.4 A human listens for clicks/artifacts at the loop boundary on at least one real project and confirms none
- [ ] 1.5 `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [ ] 1.6 Reviewed by `review-rust-core`
