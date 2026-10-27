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

### Requirement: One chord per line
The parser MUST reject a line with more than one [chord] with score_parse_error naming that line, and MUST NOT keep only the first chord.

#### Scenario: Two chords on a line
- **WHEN** a line reads `[G]Twinkle twinkle [C]little star`
- **THEN** parsing fails with score_parse_error on that line, saying the dialect allows one chord per line

### Requirement: Strict time anchors
A {t:} anchor SHALL be `minute:second.hundredth` with a two-digit second (00–59) and a two-digit hundredth; it SHALL NOT be earlier than the previous anchor or past the project's duration, and a [chord] SHALL have an anchor of its own.

#### Scenario: Malformed anchor
- **WHEN** a line holds `{t: 1:5.3}`
- **THEN** parsing fails with score_parse_error on that line

#### Scenario: Anchor and line together
- **WHEN** a line holds `{t: 0:01.00}[G]lyrics`
- **THEN** the chord G starts at 1.0 s

### Requirement: Chord windows
A chord's startSec SHALL be its line's anchor and its endSec the start of the next chord or tab block; a lyric line without a chord SHALL NOT end the current chord; the last chord SHALL end at the project's duration, given to the parser from the project's largest stems[].durationSec; the bar grid's origin SHALL be the file's first anchor.

#### Scenario: Lyric line under the same chord
- **WHEN** a chord line at 0 s is followed by a lyric-only line at 2 s and a chord line at 4 s
- **THEN** the first chord spans 0–4 s

#### Scenario: Last chord
- **WHEN** the last chord starts at 1 s in a project whose longest stem lasts 187.42 s
- **THEN** that chord ends at 187.42 s

### Requirement: Tablature staff
A tab block SHALL hold six staff lines of equal width, string 1 on top; each run of characters other than `-` and `|` SHALL be a note in the playbook's grammar on that line's string, timed by its column between the block's start and end; a fret outside 0–24, a leading zero, or a release with no earlier bend SHALL be a score_parse_error on that line.

#### Scenario: Note timing
- **WHEN** a 64-column block spans 0–6.4 s
- **THEN** a note at column 9 starts at 0.9 s

#### Scenario: Release without bend
- **WHEN** a staff line holds `7r5`
- **THEN** parsing fails with score_parse_error on that staff line
