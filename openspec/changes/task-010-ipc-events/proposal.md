# TASK-010 — IPC Events surface (playback_progress, waveform_ready, transport_state_changed, audio_error)

**Area:** Communication Bridge — Tauri IPC

## Why

The UI never decides state; it can only show what the core reports, so every state change the UI depends on needs a defined event.

## What Changes

- **Events match the events table** — The core SHALL emit playback_progress, waveform_ready, transport_state_changed and audio_error with the producer, payload and consumers in the architecture events table.
- **Audio failures are reported** — When audio output fails, the core SHALL emit audio_error with a cause and a message.

## Capabilities

### New Capabilities

- `ipc-events`: Defines the events the core emits so the UI can present state it never decides itself.

### Modified Capabilities

None.

## Impact

Communication Bridge — Tauri IPC. Status, branch, dependencies and playbooks for this change are in `openspec/board.toml`.
