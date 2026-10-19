## Purpose

Shows the stems' waveforms, the playback position and the loop markers on a timeline.

## ADDED Requirements

### Requirement: Waveform and position rendering
The timeline SHALL render peaks from waveform_ready and the playhead from playback_progress.

#### Scenario: Peaks arrive
- **WHEN** a waveform_ready event arrives for a stem
- **THEN** that stem's waveform is drawn from its peaks

### Requirement: Markers move only after confirmation
Selecting a loop SHALL send set_markers, and the markers SHALL move on screen only after the core's response.

#### Scenario: Accepted markers
- **WHEN** the core accepts set_markers
- **THEN** the markers move to the new positions

#### Scenario: Rejected markers
- **WHEN** the core rejects set_markers
- **THEN** the markers stay where they were
