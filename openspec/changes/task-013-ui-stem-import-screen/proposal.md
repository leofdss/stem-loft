# TASK-013 — Stem Import Screen (Angular)

**Area:** Angular — Stem Import Screen

## Why

Users need a screen to choose stem files, and it must leave every decision about the import to the core.

## What Changes

- **Import sends a Command immediately** — Confirming an import SHALL send the import Command immediately, with no client-side queue and no optimistic update.
- **Control disabled until the response** — The control that triggered the import SHALL stay disabled until that Command's response arrives.

## Capabilities

### New Capabilities

- `ui-stem-import`: The screen where the user picks stem files to import into the project.

### Modified Capabilities

None.

## Impact

Angular — Stem Import Screen. Status, branch, dependencies and playbooks for this change are in `openspec/board.toml`.
