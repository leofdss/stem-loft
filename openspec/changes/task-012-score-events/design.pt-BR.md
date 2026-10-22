# Design

## Context

Implementa a TASK-012 sobre o parser da TASK-011 e a Session das TASK-009/004: o núcleo publica a partitura interpretada (`score_loaded`) ou o seu erro (`score_parse_error`), e a Chord/Tab View deriva o acorde e o tempo ativos para cada posição de reprodução, como `docs/architecture.md#real-time-visual-state-angular` atribui ao lado Angular.

## Goals / Non-Goals

- Objetivo: um `score_loaded` (ou um `score_parse_error`) por interpretação, levando a partitura inteira, inclusive `tempo`/`time`/`tuning`.
- Objetivo: o recorte de acorde/tempo por tick, calculado a partir da partitura estática e de uma posição.
- Fora do escopo: emitir `playback_progress` e redesenhar a view a cada tick (TASK-010, TASK-017); um Command para associar ou trocar a partitura de um projeto (na v1 o campo `score` é definido no arquivo do projeto).

## Decisions

- **Um event sink sem Tauri no núcleo.** `core::events::EventSink` tem um método por evento; `ipc::events::TauriEvents` o implementa com o `Emitter` do Tauri. Os módulos de domínio publicam pela trait (DI manual), então o núcleo continua sem depender do Tauri e os testes registram eventos com um fake. O `AppState` agora é construído no `setup` do Tauri, onde o handle do app existe.
- **Quem produz publica.** O `Score Metadata Manager` (o produtor na tabela de eventos) carrega, interpreta e publica; a Session só pede isso quando um projeto com `score` é aberto, fora do seu lock e depois que o projeto foi instalado, então uma partitura quebrada nunca faz a abertura falhar. O `.cho` é indicado relativo à pasta do projeto, e o manager o resolve, recusando com um `score_parse_error` um nome que sai da pasta (absoluto ou com `..`).
- **Estático uma vez, recorte por tick.** O `score_loaded` leva o `Score` inteiro uma vez. A cada tick, o `chordBeatAt(score, positionSec)` do Angular devolve só `{ positionSec, activeChord, activeBeat }`: o acorde cuja janela contém a posição, e o tempo contado a partir da origem da grade a `60 / tempo` s por tempo, com o numerador de `time` tempos por compasso. Nenhum payload de tick repete a partitura.
- **Entrega com contrato primeiro.** O orquestrador escreveu o contrato (`chord-beat-stream.spec.ts`) e o lado Rust; o implementer Haiku preencheu o `chordBeatAt` (verde em 1 execução do check). As revisões levaram a resolver o caminho da partitura dentro do manager e a limpar a partitura mostrada quando um recarregamento falha.

## Risks / Trade-offs

- Um tempo dura `60 / tempo` s independentemente do denominador de `time`: o `{tempo}` é lido como tempos da unidade da fórmula de compasso. Uma partitura que quer dizer tempo de semínima em 6/8 contaria tempos de colcheia rápido demais.
- Reabrir um projeto publica a partitura de novo; nada republica quando o `.cho` muda no disco com o projeto aberto.
- Abrir um projeto sem partitura não publica nada, então a UI precisa limpar por conta própria a partitura do projeto anterior ao abrir/criar/fechar; isso é um critério da TASK-017, dona da Chord/Tab View.
