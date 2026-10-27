# Design

## Context

Implementa a superfície de Commands da TASK-009: a fila do núcleo (`docs/architecture.md#design-note-commands-queue-in-the-rust-core`), o despacho da Session sobre um projeto aberto e a ligação com a persistência adiada da TASK-002 (critério 1.6). Usa o `StemImporter::import` da TASK-003.

## Goals / Non-Goals

- Objetivo: todo Command passa por uma única fila FIFO no núcleo, fora da thread principal da janela.
- Objetivo: Commands `create_project`, `open_project` e `close_project`, para que `import_stems` e os demais tenham um projeto sobre o qual agir.
- Objetivo: Commands que mudam estado marcam o projeto como sujo; um ticker de fundo escreve depois do debounce; pause, fechar, abrir/criar e sair do app escrevem na hora.
- Fora do escopo: validar marcadores contra a duração do projeto (TASK-004), reprodução ou mixer de verdade (TASK-005, TASK-008), eventos (TASK-010, TASK-012).

## Decisions

- **A fila é um ticket lock.** `CommandQueue::run` pega um ticket ao chegar e espera a sua vez num `Mutex` + `Condvar`; a vez passa adiante mesmo se o Command der panic. É o modelo "trava, muda, solta" da nota de concorrência com justiça na ordem, sem canal nem thread worker. O mutex dela sequencia Commands e não guarda estado da aplicação, como o lock de checkpoint abaixo.
- **Handlers async, trabalho bloqueante.** Cada handler do Tauri é `async` e roda `queue.run(|| session.…)` em `spawn_blocking`, então copiar stems nunca congela a janela. A ordem é garantida a partir do momento em que o Command pega o seu ticket; o Tauri pode iniciar em qualquer ordem dois Commands enviados no mesmo instante (veja a nota de design da fila).
- **A Session guarda o estado do projeto aberto.** Ela mantém `{ path, Project }`, copia o que um módulo precisa para fora do seu lock, chama o módulo sem segurá-lo e registra uma mudança aceita (stems, loop, mixer) junto com `mark_dirty` numa única seção travada. Os padrões (um projeto vazio, os ajustes de mixer de um stem) ficam junto de `Project` em `persistence.rs`.
- **Todo Command de projeto precisa de um projeto aberto.** Sem um, ele devolve `no project is open` e o módulo não é chamado.
- **Checkpoints: token e snapshot juntos, checkpoints inteiros em série.** O `ProjectPersistence` ganhou `begin_checkpoint`/`write_checkpoint`; a Session pega o token e clona o snapshot sob o seu lock e escreve depois de soltá-lo, então nenhuma mudança é marcada como salva sem ter sido escrita. Um segundo mutex da Session põe os checkpoints inteiros em série, para que uma escrita lenta do ticker com um snapshot mais antigo não caia depois de uma definitiva mais nova.
- **Eventos definitivos.** Pause (depois que o engine pausou), fechar, abrir ou criar outro projeto (antes de ler ou criar qualquer coisa) e sair do app (pela fila, depois dos Commands em andamento) escrevem na hora. Uma falha ao salvar no pause não faz o pause falhar: o projeto continua sujo e o ticker tenta de novo.
- **Entrega com contrato primeiro.** O orquestrador escreveu a fila, a divisão do checkpoint, a ligação com o Tauri e os testes de contrato; o implementer Haiku preencheu o despacho da Session (verde em 1 execução do check). A revisão `review-rust-core` achou uma corrida entre checkpoints, um recarregamento desatualizado do projeto aberto, padrões de mixer/projeto morando na Session, um pause falhando por erro de save e o checkpoint de saída ignorando Commands em andamento; cada um foi corrigido, os dois primeiros com testes.

## Risks / Trade-offs

- O ticker de fundo acorda a cada 100 ms enquanto o app roda; uma falha de escrita persistente é registrada em todo tick.
- Dois Commands disparados no mesmo instante podem rodar em qualquer ordem; o estado pendente por botão na UI impede um mesmo controle de fazer isso.
