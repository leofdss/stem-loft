# TASK-013 — Tela de Importação de Stems (Angular)

**Área:** Angular — Stem Import Screen

## Why

Os usuários precisam de uma tela para escolher os arquivos de stems, e ela deve deixar toda decisão sobre a importação para o núcleo.

## What Changes

- **A importação envia um Command na hora** — Confirmar uma importação SHALL enviar o Command de importação na hora, sem fila no cliente e sem atualização otimista.
- **Controle desabilitado até a resposta** — O controle que disparou a importação SHALL ficar desabilitado até chegar a resposta daquele Command.

## Capabilities

### New Capabilities

- `ui-stem-import`: A tela em que o usuário escolhe os arquivos de stems para importar no projeto.

### Modified Capabilities

Nenhuma.

## Impact

Angular — Stem Import Screen. Status, branch, dependências e playbooks desta change ficam em `openspec/board.toml`.
