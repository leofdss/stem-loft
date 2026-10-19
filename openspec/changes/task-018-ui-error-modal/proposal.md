# TASK-018 — Error Modal (Angular)

**Area:** Angular — Error Modal

## Why

Errors the core reports have to reach the user clearly, without the UI trying to fix or undo anything on its own.

## What Changes

- **Errors open a modal** — audio_error and score_parse_error SHALL open the modal showing the event's cause and message.
- **Dismissing changes nothing else** — Dismissing the modal MUST NOT pause, stop or undo anything running in the core.

## Capabilities

### New Capabilities

- `ui-error-modal`: Shows errors reported by the core in a modal dialog.

### Modified Capabilities

None.

## Impact

Angular — Error Modal. Status, branch, dependencies and playbooks for this change are in `openspec/board.toml`.
