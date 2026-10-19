## Purpose

Controles de play, pause e stop que refletem o estado de transporte do núcleo.

## ADDED Requirements

### Requirement: O transporte reflete o estado do núcleo
Os controles SHALL mostrar o estado do último evento transport_state_changed e MUST NOT supor um estado antes de esse evento chegar.

#### Scenario: Apertando play
- **WHEN** o usuário aperta play
- **THEN** os controles continuam mostrando o estado anterior até o transport_state_changed informar que está tocando
