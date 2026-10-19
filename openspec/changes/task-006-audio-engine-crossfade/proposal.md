# TASK-006 — Loop crossfade at the boundary

**Area:** Rust core — Audio Engine

## Why

Short crossfade (a few ms) at the loop boundary instead of a hard cut, per the design note (`docs/architecture.md#design-note-crossfade-at-the-loop-boundary`).

## What Changes

- **Crossfade at the loop boundary** — When playback reaches endSec, the engine SHALL crossfade over a few milliseconds into startSec instead of cutting abruptly.
- **Loop start prepared in advance** — The audio at startSec MUST be prepared before playback reaches endSec, so the crossfade needs no decoding, allocation or I/O in the callback.

## Capabilities

### New Capabilities

- `loop-crossfade`: Makes the jump from the loop's end back to its start seamless, with a short crossfade instead of a hard cut.

### Modified Capabilities

None.

## Impact

Rust core — Audio Engine. Status, branch, dependencies and playbooks for this change are in `openspec/board.toml`.
