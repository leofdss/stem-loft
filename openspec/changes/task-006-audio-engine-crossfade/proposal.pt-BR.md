# TASK-006 — Crossfade na fronteira do loop

**Área:** Núcleo Rust — Audio Engine

## Why

Crossfade curto (alguns ms) na fronteira do loop em vez de um corte seco, conforme a nota de design (`docs/architecture.md#design-note-crossfade-at-the-loop-boundary`).

## What Changes

- **Crossfade na borda do loop** — Quando a reprodução chega ao endSec, o engine SHALL fazer um crossfade de poucos milissegundos para o startSec, em vez de cortar abruptamente.
- **Início do loop preparado com antecedência** — O áudio do startSec MUST estar preparado antes de a reprodução chegar ao endSec, para que o crossfade não precise de decodificação, alocação nem I/O no callback.

## Capabilities

### New Capabilities

- `loop-crossfade`: Faz a volta do fim do loop para o início ser suave, com um crossfade curto em vez de um corte seco.

### Modified Capabilities

Nenhuma.

## Impact

Núcleo Rust — Audio Engine. Status, branch, dependências e playbooks desta change ficam em `openspec/board.toml`.
