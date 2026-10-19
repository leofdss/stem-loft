## Purpose

Parses a project's extended ChordPro (.cho) file into timed chords, tablature and lyrics.

## ADDED Requirements

### Requirement: Extended ChordPro grammar
The parser SHALL accept the dialect in the chordpro-format playbook: one chord per line, {t:} time anchors, the tablature grammar, and startSec/endSec resolution.

#### Scenario: Valid score
- **WHEN** a valid .cho file is parsed
- **THEN** every chord has a resolved startSec and endSec

### Requirement: Unknown directives are ignored
The parser SHALL skip a directive it does not know and continue parsing.

#### Scenario: Unknown directive
- **WHEN** a file contains a directive the dialect doesn't define
- **THEN** the rest of the file is parsed normally

### Requirement: Parse errors are isolated
On invalid input, the parser SHALL report score_parse_error with the offending line and a message, and the rest of the application MUST keep working.

#### Scenario: Broken line
- **WHEN** a .cho file has a malformed line
- **THEN** score_parse_error names that line and playback keeps working
