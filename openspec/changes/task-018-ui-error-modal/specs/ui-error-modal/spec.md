## Purpose

Shows errors reported by the core in a modal dialog.

## ADDED Requirements

### Requirement: Errors open a modal
audio_error and score_parse_error SHALL open the modal showing the event's cause and message.

#### Scenario: Audio error
- **WHEN** an audio_error event arrives
- **THEN** the modal opens with its cause and message

### Requirement: Dismissing changes nothing else
Dismissing the modal MUST NOT pause, stop or undo anything running in the core.

#### Scenario: Dismiss during playback
- **WHEN** the user dismisses the modal while a song plays
- **THEN** playback continues unchanged
