## Purpose

Holds each stem's volume, mute and solo, and applies changes to playback in real time.

## ADDED Requirements

### Requirement: Real-time-safe mixer changes
A volume, mute or solo change SHALL reach playback through lock-free shared state; the audio callback MUST NOT block on a lock to read it.

#### Scenario: Fader moved during playback
- **WHEN** the user changes a stem's volume while the song plays
- **THEN** the level changes on the next audio block without a glitch

### Requirement: Mute and solo
Muting a stem SHALL silence it, and when any stem is soloed only soloed stems SHALL be audible.

#### Scenario: Solo one stem
- **WHEN** one stem is soloed
- **THEN** only that stem is audible until solo is cleared

### Requirement: Mixer changes persist through the checkpoint
A mixer change SHALL mark the project dirty for the debounced checkpoint instead of writing the project on each change.

#### Scenario: Dragging a fader
- **WHEN** a fader is dragged through many values
- **THEN** the project is marked dirty and written once at the next checkpoint
