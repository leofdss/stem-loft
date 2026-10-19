# TASK-017 — Chord/Tab View (Angular, somente leitura)

**Área:** Angular — Chord/Tab View

## Why

Músicos leem acordes e tablaturas enquanto tocam, então a tela precisa acompanhar a música e não pode derrubar o app quando a partitura está quebrada.

## What Changes

- **Exibição sincronizada com a reprodução** — A tela SHALL destacar o acorde e o tempo do recorte de tempo real mais recente.
- **Erros de interpretação ficam nesta tela** — Um score_parse_error SHALL colocar só esta tela num estado de erro; o resto da interface MUST continuar funcionando.

## Capabilities

### New Capabilities

- `ui-chord-tab-view`: Exibição somente leitura de acordes, tablaturas e letras em sincronia com a reprodução.

### Modified Capabilities

Nenhuma.

## Impact

Angular — Chord/Tab View. Status, branch, dependências e playbooks desta change ficam em `openspec/board.toml`.
