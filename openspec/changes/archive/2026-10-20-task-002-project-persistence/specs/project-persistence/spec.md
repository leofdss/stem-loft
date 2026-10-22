## Purpose

Loads and saves a project's state on disk without ever leaving a corrupt or half-written project file behind.

## ADDED Requirements

### Requirement: Atomic project writes
The core MUST write the project file by writing a temporary file in the same directory and renaming it over the final path, and MUST NOT write to the final path directly.

#### Scenario: Crash mid-write
- **WHEN** the process stops while a save is in progress
- **THEN** the previously saved project file is still intact and loads normally

#### Scenario: Write fails
- **WHEN** any step of a save fails, such as the disk being full
- **THEN** the save reports the error with its original cause, the previous file is intact, and no temporary file is left behind

### Requirement: Concurrent writes do not collide
Two saves of the same file at the same time SHALL each complete as a whole, leaving the file with the contents of one of them.

#### Scenario: Two saves at once
- **WHEN** two saves of the same project run concurrently
- **THEN** both succeed and the file holds one complete version

### Requirement: Debounced checkpoints
A state change SHALL mark the project dirty, and the core SHALL write it at a debounced checkpoint rather than once per Command; a definitive event such as pausing or closing SHALL write a dirty project immediately.

#### Scenario: Burst of changes
- **WHEN** many Commands change the project within one debounce window
- **THEN** the project file is written once

#### Scenario: Closing the project
- **WHEN** the project is closed with unsaved changes
- **THEN** it is written right away, without waiting for the debounce

### Requirement: Changes made during a write are kept
A change that arrives while a checkpoint is writing SHALL keep the project dirty, so the next checkpoint saves it.

#### Scenario: Change during a write
- **WHEN** a Command changes the project while a checkpoint write is in progress
- **THEN** the project is still dirty after that write and the change is saved at the next checkpoint

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
- **THEN** loading fails with an error saying it was saved by a newer version of the app, and the file is left untouched

### Requirement: Unreadable projects fail explicitly
Loading MUST end in an explicit error, never a partial project or a crash, when the file is not valid JSON, has no valid schemaVersion, or a migration fails or produces something that is not a project.

#### Scenario: Malformed file
- **WHEN** a project file is not JSON or has no valid schemaVersion
- **THEN** loading fails with a malformed-project error

#### Scenario: Broken migration
- **WHEN** a migration fails or returns something that is not a project object
- **THEN** loading fails with a migration error naming the version it started from, and the file is left untouched
