# TASK-001 — Session/State Manager & AppState wiring

**Area:** Rust core — Session/State Manager

## Why

Replace the `session.rs` stub with a real `AppState` behind a single `Arc<Mutex<AppState>>` (or at most one `Mutex` per module), and wire the manual-DI constructors for the domain modules Session routes `Commands` to.

## What Changes

- **Single shared application state** — The core MUST keep shared application state behind a single mutex, or at most one mutex per domain module, and MUST NOT use channels or actors for it.
- **Session tracks only the open project** — The session SHALL record which project is currently open and nothing else of its own.
- **Commands are routed to their owning module** — The session SHALL delegate every Command to the single module that owns that domain and SHALL return that module's result unchanged, without deciding anything itself.

## Capabilities

### New Capabilities

- `session-state`: Holds which project is open and routes every Command to the module that owns its domain, over shared state with a deliberately simple concurrency model.

### Modified Capabilities

None.

## Impact

Rust core — Session/State Manager. Status, branch, dependencies and playbooks for this change are in `openspec/board.toml`.
