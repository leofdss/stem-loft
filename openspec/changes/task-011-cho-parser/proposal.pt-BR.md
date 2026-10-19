# TASK-011 — Parser do `.cho` (Score Metadata Manager)

**Área:** Núcleo Rust — Score Metadata Manager

## Why

Acordes e tablaturas vêm de um arquivo .cho num dialeto estendido de ChordPro, e nada pode ser mostrado em sincronia antes de ele ser interpretado.

## What Changes

- **Gramática ChordPro estendida** — O parser SHALL aceitar o dialeto do playbook chordpro-format: um acorde por linha, âncoras de tempo {t:}, a gramática de tablatura e a resolução de startSec/endSec.
- **Diretivas desconhecidas são ignoradas** — O parser SHALL pular uma diretiva que não conhece e continuar a interpretação.
- **Erros de interpretação ficam isolados** — Com entrada inválida, o parser SHALL informar score_parse_error com a linha problemática e uma mensagem, e o resto da aplicação MUST continuar funcionando.

## Capabilities

### New Capabilities

- `chordpro-parsing`: Interpreta o arquivo ChordPro estendido (.cho) de um projeto em acordes, tablaturas e letras com tempo.

### Modified Capabilities

Nenhuma.

## Impact

Núcleo Rust — Score Metadata Manager. Status, branch, dependências e playbooks desta change ficam em `openspec/board.toml`.
