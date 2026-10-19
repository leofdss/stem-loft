# TASK-017 — Chord/Tab View (Angular, read-only)

**Area:** Angular — Chord/Tab View

## Why

Musicians read chords and tablature while they play, so the view must follow the song and must not take the app down when the score is broken.

## What Changes

- **Display synced to playback** — The view SHALL highlight the chord and beat from the latest real-time slice.
- **Parse errors stay in this view** — A score_parse_error SHALL put only this view into an error state; the rest of the screen MUST keep working.

## Capabilities

### New Capabilities

- `ui-chord-tab-view`: Read-only display of chords, tablature and lyrics in sync with playback.

### Modified Capabilities

None.

## Impact

Angular — Chord/Tab View. Status, branch, dependencies and playbooks for this change are in `openspec/board.toml`.
