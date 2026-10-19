## Purpose

Plays a project's stems on the audio output device continuously and without glitches.

## ADDED Requirements

### Requirement: Continuous playback
The engine SHALL play the project's stems, mixed together, on the selected output device without audible dropouts.

#### Scenario: Playing a project
- **WHEN** the user starts playback on a real output device
- **THEN** all stems play in sync with no clicks or gaps

### Requirement: Real-time callback safety
The audio output callback MUST NOT allocate memory, take a lock or perform I/O; it SHALL only read audio prepared outside it.

#### Scenario: Decoding happens elsewhere
- **WHEN** the callback needs the next block of samples
- **THEN** it reads them from a buffer the decoding stage already filled
