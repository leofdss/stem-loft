# TASK-005 — Audio Engine: cpal output + symphonia decode + rtrb pipeline

**Area:** Rust core — Audio Engine

## Why

Wire `symphonia` decoding → `rtrb` ring buffer → `cpal` callback for basic playback (no loop/crossfade yet — that's TASK-006).

## What Changes

- **Continuous playback** — The engine SHALL play the project's stems, mixed together, on the selected output device without audible dropouts.
- **Real-time callback safety** — The audio output callback MUST NOT allocate memory, take a lock or perform I/O; it SHALL only read audio prepared outside it.

## Capabilities

### New Capabilities

- `audio-playback`: Plays a project's stems on the audio output device continuously and without glitches.

### Modified Capabilities

None.

## Impact

Rust core — Audio Engine. Status, branch, dependencies and playbooks for this change are in `openspec/board.toml`.
