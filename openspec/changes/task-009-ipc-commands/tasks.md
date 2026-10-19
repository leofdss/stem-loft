# Tasks

## 1. Acceptance criteria

- [ ] 1.1 The `Commands` queue lives entirely in the Rust core; nothing in Angular queues, dedups, or reorders
- [ ] 1.2 Each `Command` handler only dispatches to the module owning that domain — no coordination logic in the handler itself
- [ ] 1.3 Unit tests cover each `Command` handler dispatching to its owning module, independent of the Tauri runtime
- [ ] 1.4 `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [ ] 1.5 Reviewed by `review-rust-core` and `review-angular-shell` (call sites)
