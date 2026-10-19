# TASK-007 — Cálculo de picos de waveform + cache em disco

**Área:** Núcleo Rust — Audio Engine / Project Persistence

## Why

A timeline precisa da forma de onda de cada stem, e decodificar os stems inteiros toda vez que um projeto abre deixaria a abertura lenta.

## What Changes

- **Picos calculados uma vez** — O núcleo SHALL calcular os picos de um stem na importação, ou na primeira abertura quando não há cache, e SHALL gravá-los em disco pelo caminho de escrita atômica.
- **Reuso do cache** — Quando existe um cache válido, o núcleo SHALL atender o waveform_ready a partir dele, sem decodificar o stem.
- **Invalidação na reimportação** — Reimportar ou substituir um stem MUST invalidar os picos em cache daquele stem.

## Capabilities

### New Capabilities

- `waveform-cache`: Fornece os picos da forma de onda de cada stem para exibição, sem decodificar o áudio de novo toda vez que um projeto abre.

### Modified Capabilities

Nenhuma.

## Impact

Núcleo Rust — Audio Engine / Project Persistence. Status, branch, dependências e playbooks desta change ficam em `openspec/board.toml`.
