## Purpose

Read-only display of chords, tablature and lyrics in sync with playback.

## ADDED Requirements

### Requirement: Display synced to playback
The view SHALL highlight the chord and beat from the latest real-time slice.

#### Scenario: Chord changes
- **WHEN** the real-time slice moves to a new chord
- **THEN** the view highlights that chord

### Requirement: Parse errors stay in this view
A score_parse_error SHALL put only this view into an error state; the rest of the screen MUST keep working.

#### Scenario: Broken score
- **WHEN** score_parse_error arrives
- **THEN** this view shows the error while transport, timeline and mixer keep working
