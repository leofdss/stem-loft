# TASK-015 — Timeline + Waveform (Angular)

**Área:** Angular — Timeline + Waveform

## Why

Praticar um trecho significa ver a música e escolher o loop nela, com marcadores que correspondem ao que o núcleo de fato aceitou.

## What Changes

- **Exibição da forma de onda e da posição** — A timeline SHALL exibir os picos do waveform_ready e o cursor de reprodução do playback_progress.
- **Marcadores só se movem após confirmação** — Selecionar um loop SHALL enviar set_markers, e os marcadores SHALL se mover na tela só depois da resposta do núcleo.

## Capabilities

### New Capabilities

- `ui-timeline`: Mostra as formas de onda dos stems, a posição de reprodução e os marcadores de loop numa timeline.

### Modified Capabilities

Nenhuma.

## Impact

Angular — Timeline + Waveform. Status, branch, dependências e playbooks desta change ficam em `openspec/board.toml`.
