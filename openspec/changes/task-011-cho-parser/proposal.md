# TASK-011 — `.cho` parser (Score Metadata Manager)

**Area:** Rust core — Score Metadata Manager

## Why

Chords and tablature come from a .cho file in an extended ChordPro dialect, and nothing can be shown in sync until it is parsed.

## What Changes

- **Extended ChordPro grammar** — The parser SHALL accept the dialect in the chordpro-format playbook: one chord per line, {t:} time anchors, the tablature grammar, and startSec/endSec resolution.
- **Unknown directives are ignored** — The parser SHALL skip a directive it does not know and continue parsing.
- **Parse errors are isolated** — On invalid input, the parser SHALL report score_parse_error with the offending line and a message, and the rest of the application MUST keep working.

## Capabilities

### New Capabilities

- `chordpro-parsing`: Parses a project's extended ChordPro (.cho) file into timed chords, tablature and lyrics.

### Modified Capabilities

None.

## Impact

Rust core — Score Metadata Manager. Status, branch, dependencies and playbooks for this change are in `openspec/board.toml`.
