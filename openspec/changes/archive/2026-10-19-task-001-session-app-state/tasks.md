# Tasks

## 1. Acceptance criteria

- [x] 1.1 `AppState` matches the concurrency-model design note (`docs/architecture.md#design-note-concurrency-model-for-shared-state`) — no channels, no actor
- [x] 1.2 `Session` holds only "which project is open"; every `Command` handler delegates to the module owning that domain, per Session as a thin router (`docs/architecture.md#design-note-session-as-a-thin-router`)
- [x] 1.3 Unit tests cover `Session` dispatching a `Command` to the correct module (a fake module records the call), without re-testing that module's own logic
- [x] 1.4 `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [x] 1.5 Reviewed by `review-rust-core`
