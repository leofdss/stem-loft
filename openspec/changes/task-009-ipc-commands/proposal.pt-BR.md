# TASK-009 — Superfície de IPC Commands

**Área:** Communication Bridge — Tauri IPC

## Why

A UI só deve enviar intenção; ordenar e executar essa intenção é papel do núcleo, para que os dois nunca discordem sobre o que aconteceu.

## What Changes

- **Fila de commands no núcleo** — Os Commands SHALL ser enfileirados e ordenados só no núcleo; a UI MUST NOT enfileirá-los, deduplicá-los nem reordená-los.
- **Handlers só despacham** — Cada handler de Command SHALL entregar o Command ao módulo dono daquele domínio, sem lógica de coordenação própria.

## Capabilities

### New Capabilities

- `ipc-commands`: Define como a UI envia intenção ao núcleo: Commands enfileirados, ordenados e executados inteiramente pelo núcleo.

### Modified Capabilities

Nenhuma.

## Impact

Communication Bridge — Tauri IPC. Status, branch, dependências e playbooks desta change ficam em `openspec/board.toml`.
