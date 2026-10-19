# TASK-001 — Session/State Manager e fiação do AppState

**Área:** Núcleo Rust — Session/State Manager

## Why

Substituir o stub `session.rs` por um `AppState` real atrás de um único `Arc<Mutex<AppState>>` (ou no máximo um `Mutex` por módulo), e conectar os construtores de injeção manual dos módulos de domínio para os quais o Session roteia `Commands`.

## What Changes

- **Estado compartilhado único da aplicação** — O núcleo MUST manter o estado compartilhado da aplicação atrás de um único mutex, ou no máximo um mutex por módulo de domínio, e MUST NOT usar canais ou atores para isso.
- **A sessão guarda só o projeto aberto** — A sessão SHALL registrar qual projeto está aberto no momento e nada mais por conta própria.
- **Commands são encaminhados ao módulo dono** — A sessão SHALL delegar cada Command ao único módulo dono daquele domínio e SHALL devolver o resultado desse módulo sem alteração, sem decidir nada por conta própria.

## Capabilities

### New Capabilities

- `session-state`: Guarda qual projeto está aberto e encaminha cada Command ao módulo dono do seu domínio, sobre um estado compartilhado com um modelo de concorrência deliberadamente simples.

### Modified Capabilities

Nenhuma.

## Impact

Núcleo Rust — Session/State Manager. Status, branch, dependências e playbooks desta change ficam em `openspec/board.toml`.
