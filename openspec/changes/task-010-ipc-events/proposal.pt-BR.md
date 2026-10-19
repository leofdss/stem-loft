# TASK-010 — Superfície de IPC Events (playback_progress, waveform_ready, transport_state_changed, audio_error)

**Área:** Communication Bridge — Tauri IPC

## Why

A UI nunca decide estado; ela só mostra o que o núcleo informa, então toda mudança de estado de que a UI depende precisa de um evento definido.

## What Changes

- **Eventos seguem a tabela de eventos** — O núcleo SHALL emitir playback_progress, waveform_ready, transport_state_changed e audio_error com o produtor, o payload e os consumidores da tabela de eventos da arquitetura.
- **Falhas de áudio são informadas** — Quando a saída de áudio falha, o núcleo SHALL emitir audio_error com uma causa e uma mensagem.

## Capabilities

### New Capabilities

- `ipc-events`: Define os eventos que o núcleo emite para que a UI apresente um estado que ela nunca decide sozinha.

### Modified Capabilities

Nenhuma.

## Impact

Communication Bridge — Tauri IPC. Status, branch, dependências e playbooks desta change ficam em `openspec/board.toml`.
