# Design

## Context

Implements TASK-012 on top of TASK-011's parser and TASK-009/004's Session: the core publishes the parsed score (`score_loaded`) or its error (`score_parse_error`), and the Chord/Tab View derives the active chord and beat for each playback position, as `docs/architecture.md#real-time-visual-state-angular` assigns to the Angular side.

## Goals / Non-Goals

- Goal: one `score_loaded` (or one `score_parse_error`) per parse, carrying the whole score including `tempo`/`time`/`tuning`.
- Goal: the per-tick chord/beat slice, computed from the static score and a position.
- Non-goal: emitting `playback_progress` and redrawing the view on each tick (TASK-010, TASK-017); a Command to attach or change a project's score (in v1 the `score` field is set in the project file).

## Decisions

- **A Tauri-free event sink in the core.** `core::events::EventSink` has one method per event; `ipc::events::TauriEvents` implements it with Tauri's `Emitter`. Domain modules publish through the trait (manual DI), so the core still has no Tauri dependency, and tests record events with a fake. `AppState` is now built in Tauri's `setup`, where the app handle exists.
- **The producer publishes.** `Score Metadata Manager` (the events table's producer) loads, parses and publishes; Session only asks it to when a project with a `score` opens, outside its lock and after the project is installed, so a broken score never fails the open. The `.cho` is named relative to the project's folder, and the manager resolves it, refusing a name that leaves the folder (absolute or `..`) with a `score_parse_error`.
- **Static once, slice per tick.** `score_loaded` carries the whole `Score` once. Each tick, Angular's `chordBeatAt(score, positionSec)` returns only `{ positionSec, activeChord, activeBeat }`: the chord whose window holds the position, and the beat counted from the grid origin at `60 / tempo` s per beat, `time`'s numerator per bar. No tick payload ever repeats the score.
- **Contract-first delivery.** The orchestrator wrote the contract (`chord-beat-stream.spec.ts`) and the Rust side; the Haiku implementer filled in `chordBeatAt` (green in 1 check run). The reviews led to resolving the score path inside the manager, and to clearing the shown score when a reload fails.

## Risks / Trade-offs

- A beat is `60 / tempo` s regardless of `time`'s denominator: `{tempo}` is read as beats of the time signature's unit. A score that means quarter-note tempo in 6/8 would count eighth-note beats too fast.
- Reopening a project republishes its score; nothing republishes when the `.cho` changes on disk while it's open.
- Opening a project without a score publishes nothing, so the UI must clear the previous project's score itself on open/create/close; that's a criterion of TASK-017, which owns the Chord/Tab View.
