## Purpose

Defines the events the core emits so the UI can present state it never decides itself.

## ADDED Requirements

### Requirement: Events match the events table
The core SHALL emit playback_progress, waveform_ready, transport_state_changed and audio_error with the producer, payload and consumers in the architecture events table.

#### Scenario: Playback running
- **WHEN** the song is playing
- **THEN** playback_progress events carry the current position

### Requirement: Audio failures are reported
When audio output fails, the core SHALL emit audio_error with a cause and a message.

#### Scenario: Device disconnected
- **WHEN** the output device is disconnected during playback
- **THEN** an audio_error event reaches the UI
