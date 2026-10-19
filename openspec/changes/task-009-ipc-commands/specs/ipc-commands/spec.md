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
