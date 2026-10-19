# TASK-009 — IPC Commands surface

**Area:** Communication Bridge — Tauri IPC

## Why

The UI must only send intent; ordering and executing that intent belongs to the core, so the two never disagree about what happened.

## What Changes

- **Core-owned command queue** — Commands SHALL be queued and ordered only in the core; the UI MUST NOT queue, deduplicate or reorder them.
- **Handlers only dispatch** — Each Command handler SHALL hand the Command to the module owning that domain, with no coordination logic of its own.

## Capabilities

### New Capabilities

- `ipc-commands`: Defines how the UI sends intent to the core: Commands queued, ordered and executed entirely by the core.

### Modified Capabilities

None.

## Impact

Communication Bridge — Tauri IPC. Status, branch, dependencies and playbooks for this change are in `openspec/board.toml`.
