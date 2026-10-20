# TASK-002 — Project Persistence: load/save, atomic writes, schemaVersion

**Area:** Rust core — Project Persistence

## Why

Implement `persistence.rs`: read/write the project `.json` per the schema (`docs/architecture.md#project-versioning`), write-to-temp+rename with no exceptions, and the three `schemaVersion` cases (same version loads, lower version runs migrations, higher version errors explicitly).

## What Changes

- **Atomic project writes** — The core MUST write the project file by writing a temporary file in the same directory and renaming it over the final path, and MUST NOT write to the final path directly.
- **Concurrent writes do not collide** — Two saves of the same file at the same time SHALL each complete as a whole, leaving the file with the contents of one of them.
- **Debounced checkpoints** — A state change SHALL mark the project dirty, and the core SHALL write it at a debounced checkpoint rather than once per Command; a definitive event such as pausing or closing SHALL write a dirty project immediately.
- **Changes made during a write are kept** — A change that arrives while a checkpoint is writing SHALL keep the project dirty, so the next checkpoint saves it.
- **Schema version handling** — When loading, the core SHALL compare the file's schemaVersion with the current one: load an equal version directly, migrate a lower one, and reject a higher one with an explicit error.
- **Unreadable projects fail explicitly** — Loading MUST end in an explicit error, never a partial project or a crash, when the file is not valid JSON, has no valid schemaVersion, or a migration fails or produces something that is not a project.

## Capabilities

### New Capabilities

- `project-persistence`: Loads and saves a project's state on disk without ever leaving a corrupt or half-written project file behind.

### Modified Capabilities

None.

## Impact

Rust core — Project Persistence. Status, branch, dependencies and playbooks for this change are in `openspec/board.toml`.
