## Purpose

The screen where the user picks stem files to import into the project.

## ADDED Requirements

### Requirement: Import sends a Command immediately
Confirming an import SHALL send the import Command immediately, with no client-side queue and no optimistic update.

#### Scenario: User confirms an import
- **WHEN** the user confirms the selected files
- **THEN** the import Command is sent and the stem list is unchanged until the core answers

### Requirement: Control disabled until the response
The control that triggered the import SHALL stay disabled until that Command's response arrives.

#### Scenario: Waiting for the core
- **WHEN** an import Command is in flight
- **THEN** the import button is disabled until its response arrives
