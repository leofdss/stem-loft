## Purpose

Provides each stem's waveform peaks for display without decoding the audio again every time a project opens.

## ADDED Requirements

### Requirement: Peaks computed once
The core SHALL compute a stem's peaks on import, or on first open when no cache exists, and SHALL write them to disk through the atomic-write path.

#### Scenario: No cache yet
- **WHEN** a project is opened and a stem has no cached peaks
- **THEN** the peaks are computed and the cache file is written

### Requirement: Cache reuse
When a valid cache exists, the core SHALL serve waveform_ready from it without decoding the stem.

#### Scenario: Reopening a project
- **WHEN** a project whose stems have cached peaks is reopened
- **THEN** waveform_ready is served from the cache and nothing is decoded

### Requirement: Invalidation on re-import
Re-importing or replacing a stem MUST invalidate that stem's cached peaks.

#### Scenario: Stem replaced
- **WHEN** a stem is replaced with a new file
- **THEN** its peaks are recomputed instead of read from the old cache

### Requirement: Self-validating cache
A cache file SHALL be used only when its format version and resolution match and it records the stem file's current size and modification time; any other cache, including a corrupt one, SHALL be recomputed rather than reported as an error.

#### Scenario: Corrupt cache
- **WHEN** a stem's cache file holds garbage
- **THEN** its peaks are recomputed and the cache rewritten

#### Scenario: Cache can't be written
- **WHEN** the cache file can't be written
- **THEN** the computed peaks are still returned
