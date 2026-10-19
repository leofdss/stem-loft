# TASK-014 — Controles de Transporte (Angular)

**Área:** Angular — Transport Controls

## Why

Play, pause e stop precisam mostrar o que o núcleo está de fato fazendo, e não o que o usuário acabou de clicar.

## What Changes

- **O transporte reflete o estado do núcleo** — Os controles SHALL mostrar o estado do último evento transport_state_changed e MUST NOT supor um estado antes de esse evento chegar.

## Capabilities

### New Capabilities

- `ui-transport`: Controles de play, pause e stop que refletem o estado de transporte do núcleo.

### Modified Capabilities

Nenhuma.

## Impact

Angular — Transport Controls. Status, branch, dependências e playbooks desta change ficam em `openspec/board.toml`.
