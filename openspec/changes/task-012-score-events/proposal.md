# TASK-012 — `score_loaded` event + real-time chord/beat slice

**Area:** Rust core / IPC — Score Metadata Manager

## Why

The chord view needs the score's header once and the active chord on every tick; sending the whole score each tick would waste the real-time channel.

## What Changes

- **Score loaded event** — The core SHALL emit score_loaded once per parse, carrying tempo, time signature and tuning from the file header.
- **Real-time chord slice** — Each real-time tick SHALL carry only the chord and beat active at the current position, never the whole score.

## Capabilities

### New Capabilities

- `score-playback-sync`: Delivers the parsed score to the UI and keeps the current chord and beat in step with playback.

### Modified Capabilities

None.

## Impact

Rust core / IPC — Score Metadata Manager. Status, branch, dependencies and playbooks for this change are in `openspec/board.toml`.
