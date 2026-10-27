# Design

## Context

Implementa o `persistence.rs` da TASK-002. As notas de arquitetura que o guiam: toda escrita em disco é atômica (`docs/architecture.md#design-note-when-project-persistence-writes`), o `.json` do projeto é versionado (`docs/architecture.md#project-versioning`), e o estado compartilhado segue o modelo de concorrência (um `Mutex` por módulo, nunca segurado durante I/O).

## Goals / Non-Goals

- Objetivo: carregar, migrar e salvar o `.json` do projeto; decidir *quando* gravar.
- Fora do objetivo: chamar o checkpoint a partir do app. Nada produz mudanças de estado ainda; ligar o `mark_dirty` e os checkpoints ao `Session` pertence ao trabalho de Commands e é um critério da TASK-009.
- Fora do objetivo: validar o conteúdo dos campos (marcadores de loop contra a duração, chaves do mixer contra os stems); isso é dos módulos donos (TASK-003, TASK-004, TASK-008).

## Decisions

- **Scheduler sem relógio.** O `SaveScheduler` recebe o `now` de quem chama e guarda só uma marca de sujo, o horário da última mudança e um contador de geração, para que toda regra de tempo seja testável sem esperas nem threads.
- **Contador de geração como token da escrita.** Uma escrita é marcada com a geração que ela cobre, não com o `Instant` da última mudança: duas mudanças podem ter o mesmo `Instant`, duas gerações não, então uma mudança feita durante uma escrita nunca é marcada como salva.
- **Lock só em volta do scheduler.** O `checkpoint` trava para decidir, solta o lock para gravar e trava de novo para concluir; o `Mutex` do módulo nunca fica preso durante I/O em disco.
- **Um temporário por escrita.** `.{nome}.tmp-{pid}-{contador}` no diretório de destino: mesmo sistema de arquivos para um `rename` atômico, e único, para que salvamentos simultâneos não sobrescrevam o temporário um do outro. Qualquer falha remove o temporário e mantém o tipo original do `io::Error`.
- **Registro de migrações checado na construção.** O `with_config` dá panic se o número de migrações não alcança a versão do schema: um erro de programação pego na inicialização, não um projeto meio migrado na hora de carregar. O carregador incrementa o `schemaVersion` depois de cada migração, então as migrações só remodelam os dados.
- **Entrega com o contrato primeiro.** Os testes de contrato foram escritos e validados antes da implementação; o primeiro rascunho da implementação veio de um modelo mais barato e foi terminado e revisado contra eles (playbook `implement-against-frozen-tests`).

## Risks / Trade-offs

- O `sync_all` cobre o arquivo temporário, mas não o diretório pai depois do `rename`: a atomicidade se mantém, mas a durabilidade do rename numa queda de energia não é garantida.
- Uma queda real do processo pode deixar um arquivo `.tmp-*` velho ao lado do projeto; nada limpa esses arquivos ainda.
