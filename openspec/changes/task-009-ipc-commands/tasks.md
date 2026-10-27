# Tasks

## 1. Acceptance criteria

- [x] 1.1 The `Commands` queue lives entirely in the Rust core; nothing in Angular queues, dedups, or reorders
- [x] 1.2 Each `Command` handler only dispatches to the module owning that domain — no coordination logic in the handler itself
- [x] 1.3 Unit tests cover each `Command` handler dispatching to its owning module, independent of the Tauri runtime
- [x] 1.4 `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [x] 1.5 Reviewed by `review-rust-core` and `review-angular-shell` (call sites)
- [x] 1.6 Every state-changing `Command` calls `ProjectPersistence::mark_dirty`, a periodic checkpoint writes after the debounce, and pausing playback or closing the project runs a definitive checkpoint — the project snapshot is cloned under the lock and written after releasing it (deferred from TASK-002)
