# TASK-004 — Loop & Marker Manager

**Área:** Núcleo Rust — Loop & Marker Manager

## Why

`set_markers`/estado da seção em loop, validado contra a duração do projeto.

## What Changes

- **Marcadores dentro da duração do projeto** — O núcleo MUST rejeitar marcadores de loop cujo endSec passe da duração do projeto (o durationSec do stem mais longo), e MUST NOT ajustá-los silenciosamente.
- **Marcadores guiam o loop na reprodução** — Os marcadores atuais SHALL ficar expostos para o audio engine, que faz o loop da reprodução entre eles.

## Capabilities

### New Capabilities

- `loop-markers`: Guarda os marcadores de início e fim do trecho em loop, validados contra a duração do projeto.

### Modified Capabilities

Nenhuma.

## Impact

Núcleo Rust — Loop & Marker Manager. Status, branch, dependências e playbooks desta change ficam em `openspec/board.toml`.
