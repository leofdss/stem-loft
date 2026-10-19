## Purpose

Brings a song's stem files into a project and normalises them on the way in, so playback never has to handle inconsistent audio.

## ADDED Requirements

### Requirement: Consistent sample rate
The importer MUST reject a stem whose sample rate differs from the stems already in the project with an explicit error, and MUST NOT resample it silently.

#### Scenario: Mismatched sample rate
- **WHEN** a 48 kHz stem is imported into a project whose stems are 44.1 kHz
- **THEN** the import is rejected with an error naming both rates

### Requirement: Mono upmix on import
The importer SHALL upmix a mono stem to stereo at import time, so playback only ever receives stereo audio.

#### Scenario: Mono stem
- **WHEN** a mono stem is imported
- **THEN** it is stored as stereo with the same signal on both channels

### Requirement: Duration from the file header
The importer SHALL read each stem's durationSec from its file header, without decoding the whole file, and store it in the project.

#### Scenario: Duration recorded
- **WHEN** a stem is imported
- **THEN** the project records its durationSec as stated in the file header
