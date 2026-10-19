# TASK-005 — Audio Engine: saída cpal + decodificação symphonia + pipeline rtrb

**Área:** Núcleo Rust — Audio Engine

## Why

Conectar decodificação `symphonia` → ring buffer `rtrb` → callback `cpal` para reprodução básica (ainda sem loop/crossfade — isso é a TASK-006).

## What Changes

- **Reprodução contínua** — O engine SHALL tocar os stems do projeto, mixados, no dispositivo de saída selecionado, sem cortes audíveis.
- **Segurança do callback de tempo real** — O callback de saída de áudio MUST NOT alocar memória, travar um lock nem fazer I/O; ele SHALL só ler áudio preparado fora dele.

## Capabilities

### New Capabilities

- `audio-playback`: Toca os stems de um projeto no dispositivo de saída de áudio de forma contínua e sem falhas.

### Modified Capabilities

Nenhuma.

## Impact

Núcleo Rust — Audio Engine. Status, branch, dependências e playbooks desta change ficam em `openspec/board.toml`.
