## Purpose

Play, pause and stop controls that reflect the core's transport state.

## ADDED Requirements

### Requirement: Transport reflects core state
The controls SHALL display the state from the latest transport_state_changed event and MUST NOT assume a state before that event arrives.

#### Scenario: Pressing play
- **WHEN** the user presses play
- **THEN** the controls keep showing the previous state until transport_state_changed reports playing
