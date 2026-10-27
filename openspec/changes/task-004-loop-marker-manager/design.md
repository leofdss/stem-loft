# Design

## Context

Implements `loop_manager.rs` for TASK-004 on top of TASK-009's Session: `set_markers` validates against the project's duration (`docs/architecture.md#consistency-across-a-projects-stems`) and the current loop is exposed to the `Audio Engine`.

## Goals / Non-Goals

- Goal: reject a loop that doesn't fit the project, never clamp it; keep the current loop where the `Audio Engine` can read it.
- Non-goal: playing the loop or its crossfade (TASK-005, TASK-006).

## Decisions

- **The project's duration comes from the project.** `Project::duration_sec` is its longest stem's `durationSec` (0 with no stems). Session reads it under its lock and passes it to `set_markers`; the Loop Manager doesn't know about stems.
- **What's rejected.** Non-finite bounds, a negative start, a start not before the end, an end past the duration, and any loop when the duration is unknown (NaN). A rejected loop leaves the current one unchanged. A project with no stems has no room for a loop.
- **One loop type.** The Loop Manager stores `Option<LoopRange>`, the same type the project file saves, instead of a second struct with the same fields; `None` means no loop.
- **Exposed to the Audio Engine.** `markers()` returns the current loop; the engine already holds the Loop Manager and copies the bounds out ahead of time, so the `cpal` callback never takes this lock.
- **Restored with the project, re-checked.** Opening a project puts its saved loop back after checking it by the same rules against the project's duration; one that no longer fits (a hand-edited file, stems that changed) is dropped, never clamped. Creating or closing a project clears the loop, before the project is released.
- **Contract-first delivery.** The orchestrator wrote the contract tests and the Session wiring; the Haiku implementer filled in `set_markers`/`restore` (green in 1 check run) and flagged the NaN-duration case, which got its own test. The `review-rust-core` review led to re-checking a restored loop and clearing the loop before releasing a project, each with a test.

## Risks / Trade-offs

- `Project.loop` and the Loop Manager hold the same fact; Session keeps them in step, relying on the Commands queue to never run two changes at once.
- `markers()` takes a lock: the `cpal` callback (TASK-005) must never call it, but copy the bounds into an atomic or double buffer from the control side.
