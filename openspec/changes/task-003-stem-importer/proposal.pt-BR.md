# TASK-003 — Stem Importer: importação manual, validação, upmix de mono

**Área:** Núcleo Rust — Stem Importer

## Why

Implementar a importação manual de stems: checagem de consistência de taxa de amostragem, upmix de mono para estéreo, e extração de `durationSec` do cabeçalho do arquivo (sem decodificação completa).

## What Changes

- **Taxa de amostragem consistente** — O importador MUST rejeitar com um erro explícito um stem cuja taxa de amostragem difere dos stems já presentes no projeto, e MUST NOT reamostrá-lo silenciosamente.
- **Upmix de mono na importação** — O importador SHALL fazer o upmix de um stem mono para estéreo no momento da importação, para que a reprodução só receba áudio estéreo.
- **Duração lida do cabeçalho** — O importador SHALL ler o durationSec de cada stem do cabeçalho do arquivo, sem decodificar o arquivo inteiro, e guardá-lo no projeto.

## Capabilities

### New Capabilities

- `stem-import`: Traz os arquivos de stems de uma música para o projeto e os normaliza na entrada, para que a reprodução nunca tenha que lidar com áudio inconsistente.

### Modified Capabilities

Nenhuma.

## Impact

Núcleo Rust — Stem Importer. Status, branch, dependências e playbooks desta change ficam em `openspec/board.toml`.
