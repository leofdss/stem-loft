# TASK-007 — Waveform peak computation + disk cache

**Area:** Rust core — Audio Engine / Project Persistence

## Why

The timeline needs every stem's waveform, and decoding whole stems on every project open would make opening a project slow.

## What Changes

- **Peaks computed once** — The core SHALL compute a stem's peaks on import, or on first open when no cache exists, and SHALL write them to disk through the atomic-write path.
- **Cache reuse** — When a valid cache exists, the core SHALL serve waveform_ready from it without decoding the stem.
- **Invalidation on re-import** — Re-importing or replacing a stem MUST invalidate that stem's cached peaks.

## Capabilities

### New Capabilities

- `waveform-cache`: Provides each stem's waveform peaks for display without decoding the audio again every time a project opens.

### Modified Capabilities

None.

## Impact

Rust core — Audio Engine / Project Persistence. Status, branch, dependencies and playbooks for this change are in `openspec/board.toml`.
