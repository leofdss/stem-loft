# TASK-015 — Timeline + Waveform (Angular)

**Area:** Angular — Timeline + Waveform

## Why

Practising a section means seeing the song and choosing the loop on it, with markers that match what the core actually accepted.

## What Changes

- **Waveform and position rendering** — The timeline SHALL render peaks from waveform_ready and the playhead from playback_progress.
- **Markers move only after confirmation** — Selecting a loop SHALL send set_markers, and the markers SHALL move on screen only after the core's response.

## Capabilities

### New Capabilities

- `ui-timeline`: Shows the stems' waveforms, the playback position and the loop markers on a timeline.

### Modified Capabilities

None.

## Impact

Angular — Timeline + Waveform. Status, branch, dependencies and playbooks for this change are in `openspec/board.toml`.
