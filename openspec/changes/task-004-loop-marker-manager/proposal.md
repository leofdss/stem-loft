# TASK-004 — Loop & Marker Manager

**Area:** Rust core — Loop & Marker Manager

## Why

`set_markers`/state for the looped section, validated against the project's duration.

## What Changes

- **Markers within the project duration** — The core MUST reject loop markers whose endSec is beyond the project's duration (the longest stem's durationSec), and MUST NOT clamp them silently.
- **Markers drive loop playback** — The current markers SHALL be exposed to the audio engine, which loops playback between them.

## Capabilities

### New Capabilities

- `loop-markers`: Keeps the start and end markers of the section being looped, validated against the project's length.

### Modified Capabilities

None.

## Impact

Rust core — Loop & Marker Manager. Status, branch, dependencies and playbooks for this change are in `openspec/board.toml`.
