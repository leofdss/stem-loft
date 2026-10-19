# TASK-012 — Evento `score_loaded` + fatia de acorde/batida em tempo real

**Área:** Núcleo Rust / IPC — Score Metadata Manager

## Why

A tela de acordes precisa do cabeçalho da partitura uma vez e do acorde ativo a cada tick; mandar a partitura inteira a cada tick desperdiçaria o canal de tempo real.

## What Changes

- **Evento de partitura carregada** — O núcleo SHALL emitir score_loaded uma vez por interpretação, com o andamento, a fórmula de compasso e a afinação do cabeçalho do arquivo.
- **Recorte de acordes em tempo real** — Cada tick de tempo real SHALL trazer só o acorde e o tempo ativos na posição atual, nunca a partitura inteira.

## Capabilities

### New Capabilities

- `score-playback-sync`: Entrega a partitura interpretada à UI e mantém o acorde e o tempo atuais acompanhando a reprodução.

### Modified Capabilities

Nenhuma.

## Impact

Núcleo Rust / IPC — Score Metadata Manager. Status, branch, dependências e playbooks desta change ficam em `openspec/board.toml`.
