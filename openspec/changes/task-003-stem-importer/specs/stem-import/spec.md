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

### Requirement: Supported stems
The importer SHALL accept WAV stems in integer PCM or IEEE float, plain or WAVE_FORMAT_EXTENSIBLE, mono or stereo, and MUST reject any other file, or a stem with more than two channels, with an explicit error.

#### Scenario: Four-channel stem
- **WHEN** a four-channel WAV is imported
- **THEN** the import is rejected with an error naming the channel count

### Requirement: All-or-nothing import
The importer SHALL check every file of an import before writing any of them, and MUST NOT leave part of a rejected import in the project folder.

#### Scenario: One bad file among good ones
- **WHEN** an import holds a valid 44.1 kHz stem and a 48 kHz stem
- **THEN** nothing is written to the project folder

### Requirement: Stems never overwrite existing files
Each imported stem SHALL be written atomically to `<id>.wav` in the project folder, with an id derived from its file name and suffixed so it clashes with no stem of the project and no file already in the folder.

#### Scenario: Same name twice
- **WHEN** `Guitar.wav` is imported into a project that already has a `guitar` stem
- **THEN** it is stored as `guitar-2.wav` and the existing file is untouched
