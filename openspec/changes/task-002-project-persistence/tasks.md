# Tasks

## 1. Acceptance criteria

- [x] 1.1 Every write goes through a temp file + `rename`, never a direct write to the final path
- [x] 1.2 Writes happen at a debounce/checkpoint, not once per `Command` (see design note (`docs/architecture.md#design-note-when-project-persistence-writes`))
- [x] 1.3 All three `schemaVersion` cases are covered by a test
- [x] 1.4 A simulated crash mid-write leaves the previous file intact (test)
- [x] 1.5 `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [x] 1.6 Reviewed by `review-rust-core`
