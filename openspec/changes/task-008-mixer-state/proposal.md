# TASK-008 — Mixer state (volume/mute/solo), real-time-safe propagation

**Area:** Rust core — Audio Engine

## Why

Musicians need to change each stem's level, mute and solo while the song plays, without glitches and without hammering the disk.

## What Changes

- **Real-time-safe mixer changes** — A volume, mute or solo change SHALL reach playback through lock-free shared state; the audio callback MUST NOT block on a lock to read it.
- **Mute and solo** — Muting a stem SHALL silence it, and when any stem is soloed only soloed stems SHALL be audible.
- **Mixer changes persist through the checkpoint** — A mixer change SHALL mark the project dirty for the debounced checkpoint instead of writing the project on each change.

## Capabilities

### New Capabilities

- `mixer`: Holds each stem's volume, mute and solo, and applies changes to playback in real time.

### Modified Capabilities

None.

## Impact

Rust core — Audio Engine. Status, branch, dependencies and playbooks for this change are in `openspec/board.toml`.
