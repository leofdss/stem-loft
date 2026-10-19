# TASK-016 — Stem Mixer (Angular)

**Area:** Angular — Stem Mixer

## Why

Each stem needs volume, mute and solo controls that pass every movement straight to the core, which owns timing and persistence.

## What Changes

- **Faders send mixer Commands directly** — Each control change SHALL send the mixer Command with no client-side queue, debounce or deduplication.
- **No disk write per movement** — Dragging a fader MUST NOT cause one project write per movement; persistence follows the core's debounced checkpoint.

## Capabilities

### New Capabilities

- `ui-mixer`: Per-stem volume, mute and solo controls.

### Modified Capabilities

None.

## Impact

Angular — Stem Mixer. Status, branch, dependencies and playbooks for this change are in `openspec/board.toml`.
