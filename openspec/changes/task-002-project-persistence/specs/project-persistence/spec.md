## Purpose

Loads and saves a project's state on disk without ever leaving a corrupt or half-written project file behind.

## ADDED Requirements

### Requirement: Atomic project writes
The core MUST write the project file by writing a temporary file and renaming it over the final path, and MUST NOT write to the final path directly.

#### Scenario: Crash mid-write
- **WHEN** the process stops while a save is in progress
- **THEN** the previously saved project file is still intact and loads normally

### Requirement: Debounced checkpoints
A state change SHALL mark the project dirty, and the core SHALL write it at a debounced checkpoint rather than once per Command.

#### Scenario: Burst of changes
- **WHEN** many Commands change the project within one debounce window
- **THEN** the project file is written once

### Requirement: Schema version handling
When loading, the core SHALL compare the file's schemaVersion with the current one: load an equal version directly, migrate a lower one, and reject a higher one with an explicit error.

#### Scenario: Same version
- **WHEN** a project with the current schemaVersion is opened
- **THEN** it loads without migration

#### Scenario: Older version
- **WHEN** a project with a lower schemaVersion is opened
- **THEN** the migrations run and the project loads at the current version

#### Scenario: Newer version
- **WHEN** a project with a higher schemaVersion is opened
- **THEN** loading fails with an explicit error and the file is left untouched
