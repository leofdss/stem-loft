# Design

## Context

Implements the Commands surface for TASK-009: the core-owned queue (`docs/architecture.md#design-note-commands-queue-in-the-rust-core`), Session dispatch over an open project, and the persistence wiring deferred from TASK-002 (criterion 1.6). Builds on TASK-003's `StemImporter::import`.

## Goals / Non-Goals

- Goal: every Command runs through one FIFO queue in the core, off the window's main thread.
- Goal: `create_project`, `open_project` and `close_project` Commands, so `import_stems` and the rest have a project to act on.
- Goal: state-changing Commands mark the project dirty; a background ticker writes after the debounce; pause, close, open/create and quitting write right away.
- Non-goal: validating markers against the project's duration (TASK-004), real playback or mixer behavior (TASK-005, TASK-008), events (TASK-010, TASK-012).

## Decisions

- **The queue is a ticket lock.** `CommandQueue::run` takes a ticket on arrival and waits for its turn on a `Mutex` + `Condvar`; the turn passes on even if the Command panics. That's the "lock, change, release" model of the concurrency note with fairness added, and no channel or worker thread. Its mutex sequences Commands and guards no application state, like the checkpoint lock below.
- **Async handlers, blocking work.** Each Tauri handler is `async` and runs `queue.run(|| session.…)` on `spawn_blocking`, so copying stems never freezes the window. Order is guaranteed from the moment a Command takes its ticket; Tauri may start two Commands sent in the same instant in either order (see the queue design note).
- **Session owns the open project's state.** It holds `{ path, Project }`, copies what a module needs out of its lock, calls the module without holding it, and records an accepted change (stems, loop, mixer) together with `mark_dirty` in one locked section. Defaults (an empty project, a stem's mixer settings) live next to `Project` in `persistence.rs`.
- **Every project Command needs an open project.** Without one it returns `no project is open` and the module isn't called.
- **Checkpoints: token and snapshot together, whole checkpoints serialized.** `ProjectPersistence` gained `begin_checkpoint`/`write_checkpoint`; Session takes the token and clones the snapshot under its lock and writes after releasing it, so no change is ever marked saved without being written. A second Session mutex serializes whole checkpoints, so a slow ticker write of an older snapshot can't land after a newer definitive one.
- **Definitive events.** Pause (after the engine paused), close, opening or creating another project (before anything is read or created), and quitting (through the queue, after in-flight Commands) write right away. A failed save on pause doesn't fail the pause: the project stays dirty and the ticker retries.
- **Contract-first delivery.** The orchestrator wrote the queue, the checkpoint split, the Tauri wiring and the contract tests; the Haiku implementer filled in Session's dispatch (green in 1 check run). The `review-rust-core` review found a checkpoint race, a stale reload of the open project, mixer/project defaults living in Session, a pause failing on a save error and the quit checkpoint skipping in-flight Commands; each was fixed, the first two with tests.

## Risks / Trade-offs

- The background ticker wakes every 100 ms while the app runs; a persistent write failure is logged on every tick.
- Two Commands fired within the same instant may run in either order; the UI's per-button pending state keeps one control from doing that.
