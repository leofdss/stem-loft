## Purpose

Per-stem volume, mute and solo controls.

## ADDED Requirements

### Requirement: Faders send mixer Commands directly
Each control change SHALL send the mixer Command with no client-side queue, debounce or deduplication.

#### Scenario: Moving a fader
- **WHEN** the user moves a volume fader
- **THEN** a mixer Command is sent for the new value

### Requirement: No disk write per movement
Dragging a fader MUST NOT cause one project write per movement; persistence follows the core's debounced checkpoint.

#### Scenario: Long drag
- **WHEN** a fader is dragged across its whole range
- **THEN** the project is written at most once per checkpoint
