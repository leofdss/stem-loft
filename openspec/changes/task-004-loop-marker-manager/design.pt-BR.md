# Design

## Context

Implementa `loop_manager.rs` para a TASK-004 sobre a Session da TASK-009: `set_markers` valida contra a duração do projeto (`docs/architecture.md#consistency-across-a-projects-stems`) e o loop atual fica exposto ao `Audio Engine`.

## Goals / Non-Goals

- Objetivo: rejeitar um loop que não cabe no projeto, nunca recortá-lo; manter o loop atual onde o `Audio Engine` possa lê-lo.
- Fora do escopo: tocar o loop ou o seu crossfade (TASK-005, TASK-006).

## Decisions

- **A duração do projeto vem do projeto.** `Project::duration_sec` é o `durationSec` do stem mais longo (0 sem stems). A Session lê isso sob o seu lock e passa para `set_markers`; o Loop Manager não sabe de stems.
- **O que é rejeitado.** Limites não finitos, início negativo, início que não vem antes do fim, fim depois da duração, e qualquer loop quando a duração é desconhecida (NaN). Um loop rejeitado deixa o atual como está. Um projeto sem stems não tem espaço para um loop.
- **Um único tipo de loop.** O Loop Manager guarda `Option<LoopRange>`, o mesmo tipo que o arquivo do projeto salva, em vez de uma segunda struct com os mesmos campos; `None` quer dizer sem loop.
- **Exposto ao Audio Engine.** `markers()` devolve o loop atual; o engine já guarda o Loop Manager e copia os limites antes, então o callback do `cpal` nunca pega este lock.
- **Restaurado com o projeto, conferido de novo.** Abrir um projeto devolve o loop salvo depois de conferi-lo pelas mesmas regras contra a duração do projeto; um que não cabe mais (um arquivo editado à mão, stems que mudaram) é descartado, nunca recortado. Criar ou fechar um projeto limpa o loop, antes de o projeto ser solto.
- **Entrega com contrato primeiro.** O orquestrador escreveu os testes de contrato e a ligação com a Session; o implementer Haiku preencheu `set_markers`/`restore` (verde em 1 execução do check) e apontou o caso da duração NaN, que ganhou o seu próprio teste. A revisão `review-rust-core` levou a conferir de novo um loop restaurado e a limpar o loop antes de soltar um projeto, cada um com teste.

## Risks / Trade-offs

- `Project.loop` e o Loop Manager guardam o mesmo fato; a Session os mantém em sincronia, contando com a fila de Commands para nunca rodar duas mudanças ao mesmo tempo.
- `markers()` pega um lock: o callback do `cpal` (TASK-005) nunca deve chamá-lo, e sim copiar os limites para uma variável atômica ou double buffer pelo lado de controle.
