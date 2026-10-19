## Purpose

Holds which project is open and routes every Command to the module that owns its domain, over shared state with a deliberately simple concurrency model.

## ADDED Requirements

### Requirement: Single shared application state
The core MUST keep shared application state behind a single mutex, or at most one mutex per domain module, and MUST NOT use channels or actors for it.

#### Scenario: Concurrent commands
- **WHEN** two Commands for different modules arrive at the same time
- **THEN** each one locks only its owning module's state, applies its change and releases it, with no message channel in between

### Requirement: Session tracks only the open project
The session SHALL record which project is currently open and nothing else of its own.

#### Scenario: Fresh session
- **WHEN** the application starts
- **THEN** no project is open

### Requirement: Commands are routed to their owning module
The session SHALL delegate every Command to the single module that owns that domain and SHALL return that module's result unchanged, without deciding anything itself.

#### Scenario: Stem import is routed
- **WHEN** an import_stems Command is dispatched
- **THEN** only the Stem Importer receives it

#### Scenario: Module error is passed through
- **WHEN** the owning module returns an error
- **THEN** the session returns that same error unchanged
