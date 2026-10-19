# session-state Specification

## Purpose
Guarda qual projeto está aberto e encaminha cada Command ao módulo dono do seu domínio, sobre um estado compartilhado com um modelo de concorrência deliberadamente simples.

## Requirements

### Requirement: Estado compartilhado único da aplicação
O núcleo MUST manter o estado compartilhado da aplicação atrás de um único mutex, ou no máximo um mutex por módulo de domínio, e MUST NOT usar canais ou atores para isso.

#### Scenario: Commands concorrentes
- **WHEN** dois Commands para módulos diferentes chegam ao mesmo tempo
- **THEN** cada um trava só o estado do módulo dono, aplica a mudança e libera, sem nenhum canal de mensagens no meio

### Requirement: A sessão guarda só o projeto aberto
A sessão SHALL registrar qual projeto está aberto no momento e nada mais por conta própria.

#### Scenario: Sessão nova
- **WHEN** a aplicação inicia
- **THEN** nenhum projeto está aberto

### Requirement: Commands são encaminhados ao módulo dono
A sessão SHALL delegar cada Command ao único módulo dono daquele domínio e SHALL devolver o resultado desse módulo sem alteração, sem decidir nada por conta própria.

#### Scenario: Importação de stems é encaminhada
- **WHEN** um Command import_stems é despachado
- **THEN** só o Stem Importer o recebe

#### Scenario: Erro do módulo é repassado
- **WHEN** o módulo dono devolve um erro
- **THEN** a sessão devolve esse mesmo erro sem alteração
