# TASK-008 — Estado do mixer (volume/mute/solo), propagação segura para tempo real

**Área:** Núcleo Rust — Audio Engine

## Why

Músicos precisam mudar o volume, o mute e o solo de cada stem enquanto a música toca, sem falhas e sem martelar o disco.

## What Changes

- **Mudanças de mixer seguras para tempo real** — Uma mudança de volume, mute ou solo SHALL chegar à reprodução por estado compartilhado sem lock; o callback de áudio MUST NOT bloquear num lock para lê-la.
- **Mute e solo** — Mutar um stem SHALL silenciá-lo, e quando algum stem está em solo só os stems em solo SHALL ser audíveis.
- **Mudanças do mixer persistem pelo checkpoint** — Uma mudança no mixer SHALL marcar o projeto como sujo para o checkpoint com debounce, em vez de gravar o projeto a cada mudança.

## Capabilities

### New Capabilities

- `mixer`: Guarda o volume, o mute e o solo de cada stem e aplica as mudanças na reprodução em tempo real.

### Modified Capabilities

Nenhuma.

## Impact

Núcleo Rust — Audio Engine. Status, branch, dependências e playbooks desta change ficam em `openspec/board.toml`.
