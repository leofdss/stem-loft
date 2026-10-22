## Purpose

Delivers the parsed score to the UI and keeps the current chord and beat in step with playback.

## ADDED Requirements

### Requirement: Score loaded event
The core SHALL emit score_loaded once per parse, carrying tempo, time signature and tuning from the file header.

#### Scenario: Score parsed
- **WHEN** a .cho file is parsed successfully
- **THEN** one score_loaded event carries its tempo, time and tuning

### Requirement: Real-time chord slice
Each real-time tick SHALL carry only the chord and beat active at the current position, never the whole score.

#### Scenario: Position inside a chord
- **WHEN** playback is at a positionSec inside a chord's range
- **THEN** the tick payload names that chord and the current beat

### Requirement: Score published when its project opens
Opening a project that names a score SHALL load it from the project's folder and publish exactly one score_loaded or one score_parse_error; a broken score MUST NOT fail the open, and a failed load SHALL clear the score the UI shows.

#### Scenario: Broken score on open
- **WHEN** a project whose .cho has a malformed line is opened
- **THEN** the project opens, one score_parse_error names the line, and no previous score stays on screen

### Requirement: Beat grid from the score header
The active beat SHALL be counted from the score's first time anchor at 60 / tempo seconds per beat with the time signature's numerator beats per bar, and SHALL be absent before that anchor or when tempo, time or an anchor is missing.

#### Scenario: Second bar
- **WHEN** an 80 BPM, 4/4 score starting at 0 s is at 3 s
- **THEN** the active beat is bar 2, beat 1, from 3 s to 3.75 s
