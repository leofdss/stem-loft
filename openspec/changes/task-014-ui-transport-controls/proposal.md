# TASK-014 — Transport Controls (Angular)

**Area:** Angular — Transport Controls

## Why

Play, pause and stop must show what the core is actually doing, not what the user just clicked.

## What Changes

- **Transport reflects core state** — The controls SHALL display the state from the latest transport_state_changed event and MUST NOT assume a state before that event arrives.

## Capabilities

### New Capabilities

- `ui-transport`: Play, pause and stop controls that reflect the core's transport state.

### Modified Capabilities

None.

## Impact

Angular — Transport Controls. Status, branch, dependencies and playbooks for this change are in `openspec/board.toml`.
