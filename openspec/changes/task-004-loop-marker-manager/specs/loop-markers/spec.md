## Purpose

Keeps the start and end markers of the section being looped, validated against the project's length.

## ADDED Requirements

### Requirement: Markers within the project duration
The core MUST reject loop markers whose endSec is beyond the project's duration (the longest stem's durationSec), and MUST NOT clamp them silently.

#### Scenario: Markers inside the song
- **WHEN** markers with endSec within the project duration are set
- **THEN** they are accepted and become the current loop

#### Scenario: End beyond the song
- **WHEN** markers with endSec beyond the project duration are set
- **THEN** they are rejected with an error and the previous markers stay in place

### Requirement: Markers drive loop playback
The current markers SHALL be exposed to the audio engine, which loops playback between them.

#### Scenario: New markers during playback
- **WHEN** markers are changed while the song is playing
- **THEN** the next loop iteration uses the new markers

### Requirement: Well-formed loops
The core MUST reject a loop whose bounds aren't finite numbers, whose start is negative or not before its end, or set while the project's duration is unknown, leaving the current loop unchanged.

#### Scenario: Start after end
- **WHEN** markers with startSec at or after endSec are set
- **THEN** they are rejected and the previous markers stay in place

### Requirement: The loop follows the open project
Opening a project SHALL restore the loop saved in it, and creating or closing a project SHALL clear the current loop.

#### Scenario: Reopening a project
- **WHEN** a project saved with a loop is opened
- **THEN** that loop is the current loop
