# TASK-018 — Modal de Erro (Angular)

**Área:** Angular — Error Modal

## Why

Os erros que o núcleo informa precisam chegar claramente ao usuário, sem a UI tentar corrigir ou desfazer nada por conta própria.

## What Changes

- **Erros abrem um modal** — audio_error e score_parse_error SHALL abrir o modal mostrando a causa e a mensagem do evento.
- **Fechar não muda mais nada** — Fechar o modal MUST NOT pausar, parar nem desfazer nada que esteja rodando no núcleo.

## Capabilities

### New Capabilities

- `ui-error-modal`: Mostra num diálogo modal os erros informados pelo núcleo.

### Modified Capabilities

Nenhuma.

## Impact

Angular — Error Modal. Status, branch, dependências e playbooks desta change ficam em `openspec/board.toml`.
