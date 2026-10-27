## Purpose

Defines how the UI sends intent to the core: Commands queued, ordered and executed entirely by the core.

## ADDED Requirements

### Requirement: Core-owned command queue
Commands SHALL be queued and ordered only in the core; the UI MUST NOT queue, deduplicate or reorder them.

#### Scenario: Two quick commands
- **WHEN** the UI sends two Commands in quick succession
- **THEN** the core runs both, in the order they arrived

### Requirement: Handlers only dispatch
Each Command handler SHALL hand the Command to the module owning that domain, with no coordination logic of its own.

#### Scenario: Set markers
- **WHEN** a set_markers Command arrives
- **THEN** its handler passes it to the loop marker module and returns that module's result

### Requirement: Commands need an open project
Every Command that acts on a project SHALL fail with "no project is open" when none is, without calling its module; create_project and open_project SHALL open one, and close_project SHALL close it.

#### Scenario: Import with nothing open
- **WHEN** import_stems arrives and no project is open
- **THEN** it fails with "no project is open" and the Stem Importer is not called

#### Scenario: Create never overwrites
- **WHEN** create_project targets a folder that already holds a project file
- **THEN** it fails, the existing file is untouched and the current project stays open

### Requirement: Changes are saved at checkpoints
A successful state-changing Command SHALL mark the open project dirty; the core SHALL write it after the debounce, and pausing, closing, opening or creating another project, and quitting SHALL write it right away; an older snapshot MUST NOT overwrite a newer one on disk.

#### Scenario: Fader drag
- **WHEN** several set_stem_volume Commands arrive within the debounce
- **THEN** the project file is written once, with the last volume

#### Scenario: Reopening the open project
- **WHEN** open_project targets the project that's already open with unsaved changes
- **THEN** those changes are saved first and the reopened project includes them
