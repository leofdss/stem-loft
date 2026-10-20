# Design

## Context

Implements `persistence.rs` for TASK-002. The architecture notes that drive it: every disk write is atomic (`docs/architecture.md#design-note-when-project-persistence-writes`), the project `.json` is versioned (`docs/architecture.md#project-versioning`), and shared state follows the concurrency model (one `Mutex` per module, never held across I/O).

## Goals / Non-Goals

- Goal: load, migrate and save the project `.json`; decide *when* to write.
- Non-goal: calling the checkpoint from the app. Nothing produces state changes yet; wiring `mark_dirty` and the checkpoints into `Session` belongs with the Commands work and is a criterion of TASK-009.
- Non-goal: validating field contents (loop markers against the duration, mixer keys against stems); those belong to the modules that own them (TASK-003, TASK-004, TASK-008).

## Decisions

- **Scheduler without a clock.** `SaveScheduler` takes `now` from the caller and holds only a dirty flag, the last change time and a generation counter, so every timing rule is testable without sleeping or threads.
- **Generation counter as the write token.** A write is tagged with the generation it covers, not the `Instant` of the last change: two changes can share an `Instant`, two generations can't, so a change during a write is never marked clean.
- **Lock only around the scheduler.** `checkpoint` locks to decide, releases the lock to write, and locks again to finish; the module's `Mutex` is never held during disk I/O.
- **Temp file per write.** `.{name}.tmp-{pid}-{counter}` in the destination directory: same file system for an atomic `rename`, and unique, so concurrent saves can't overwrite each other's temp file. Any failure removes the temp file and keeps the original `io::Error` kind.
- **Migrations registry checked at construction.** `with_config` panics if the number of migrations can't reach the schema version: a programming error caught at startup, not a half-migrated project at load time. The loader bumps `schemaVersion` after each migration, so migrations only reshape data.
- **Contract-first delivery.** The contract tests were written and validated before the implementation; the first draft of the implementation came from a cheaper model and was finished and reviewed against them (`implement-against-frozen-tests` playbook).

## Risks / Trade-offs

- `sync_all` covers the temp file but not the parent directory after `rename`: atomicity holds, durability of the rename across a power loss is not guaranteed.
- A real process crash can leave a stale `.tmp-*` file next to the project; nothing sweeps them yet.
