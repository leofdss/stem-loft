# Tasks

## 1. Acceptance criteria

- [ ] 1.1 Each event matches the events table (`docs/architecture.md#events-table`) exactly (producer, payload, consumers)
- [ ] 1.2 `audio_error` is actually reachable in manual testing (e.g. disconnect the output device)
- [ ] 1.3 Unit tests cover each event's payload shape against the events table (`docs/architecture.md#events-table`)
- [ ] 1.4 `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [ ] 1.5 Reviewed by `review-rust-core`
