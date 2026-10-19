# TASK-003 — Stem Importer: manual import, validation, mono upmix

**Area:** Rust core — Stem Importer

## Why

Implement manual stem import: sample-rate consistency check, mono→stereo upmix, and `durationSec` extraction from the file header (no full decode).

## What Changes

- **Consistent sample rate** — The importer MUST reject a stem whose sample rate differs from the stems already in the project with an explicit error, and MUST NOT resample it silently.
- **Mono upmix on import** — The importer SHALL upmix a mono stem to stereo at import time, so playback only ever receives stereo audio.
- **Duration from the file header** — The importer SHALL read each stem's durationSec from its file header, without decoding the whole file, and store it in the project.

## Capabilities

### New Capabilities

- `stem-import`: Brings a song's stem files into a project and normalises them on the way in, so playback never has to handle inconsistent audio.

### Modified Capabilities

None.

## Impact

Rust core — Stem Importer. Status, branch, dependencies and playbooks for this change are in `openspec/board.toml`.
