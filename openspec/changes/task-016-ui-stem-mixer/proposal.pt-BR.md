# TASK-016 — Mixer de Stems (Angular)

**Área:** Angular — Stem Mixer

## Why

Cada stem precisa de controles de volume, mute e solo que passem cada movimento direto ao núcleo, que é dono do tempo e da persistência.

## What Changes

- **Faders enviam Commands do mixer direto** — Cada mudança de controle SHALL enviar o Command do mixer sem fila, debounce nem deduplicação no cliente.
- **Nenhuma escrita em disco por movimento** — Arrastar um fader MUST NOT causar uma escrita do projeto por movimento; a persistência segue o checkpoint com debounce do núcleo.

## Capabilities

### New Capabilities

- `ui-mixer`: Controles de volume, mute e solo por stem.

### Modified Capabilities

Nenhuma.

## Impact

Angular — Stem Mixer. Status, branch, dependências e playbooks desta change ficam em `openspec/board.toml`.
