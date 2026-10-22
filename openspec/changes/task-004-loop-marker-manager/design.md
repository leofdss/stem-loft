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
- **Restored with the project.** Opening a project puts its saved loop back (it was validated when set); creating or closing one clears it.
- **Contract-first delivery.** The orchestrator wrote the contract tests and the Session wiring; the Haiku implementer filled in `set_markers`/`restore` (green in 1 check run) and flagged the NaN-duration case, which got its own test.

## Risks / Trade-offs

- A saved loop isn't re-checked on open: if the project file was edited by hand to a loop past the stems, it's restored as is until the next `set_markers`.
