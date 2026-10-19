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
