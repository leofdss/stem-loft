---
name: add-ipc-contract
description: Roteiro para adicionar ou alterar um Command (Angular → Rust) ou Event (Rust → Angular) no StemLoft, respeitando as decisões já fechadas em docs/doc.md (fila no núcleo, Sessão como roteador fino, Signals sem NgRx, sem atualização otimista). Use sempre que for expor uma nova ação de UI, um novo estado a reportar, ou perguntar "onde entra esse Command/Event".
---

# Adicionar um Command ou Event

Este é o ponto de entrada mais comum de trabalho no StemLoft: quase toda
feature nova passa por adicionar um `Command` (Angular → Rust) e/ou um
`Event` (Rust → Angular). As decisões de design que governam isso já estão
fechadas — este skill existe pra você não precisar re-decidir nada, só seguir
o roteiro.

Fonte da verdade: [`docs/doc.md`](../../../docs/doc.md), seções "Ponte de
Comunicação — Tauri IPC" e notas de design ligadas a ela. Se algo aqui
divergir do `doc.md`, **o `doc.md` vale**, e você deve atualizar este skill —
não o contrário.

## Antes de começar: onde a lógica mora

**Nunca** ponha lógica de decisão no Angular. Angular só apresenta estado e
envia intenção — nada de fila, dedup, ordenação ou regra de negócio do lado
dele ([nota de design](../../../docs/doc.md#nota-de-design-fila-de-commands-no-núcleo-rust)).

**Nunca** ponha a lógica nova no `Gerenciador de Sessão / Estado`. Sessão é
um roteador fino — só sabe "qual projeto está aberto" e despacha pro módulo
dono do domínio
([nota de design](../../../docs/doc.md#nota-de-design-sessão-como-roteador-fino)).
Pergunta pra decidir onde o handler vive: **qual módulo de domínio é dono
dessa informação?** (`Motor de Áudio`, `Gerenciador de Loops`, `Importador de
Stems`, `Persistência de Projetos`, `Gerenciador de Metadados de Partitura`).
Se a resposta for "mais de um, e alguém precisa coordenar com lógica
própria", isso é sinal de que falta um módulo novo — não que a Sessão deve
crescer.

## Passo a passo: adicionar um Command

1. **Escolha o nome** do Command em português, verbo no infinitivo/imperativo
   consistente com os já existentes (`importar_stems`, `definir_marcadores`).
2. **Handler fica no módulo de domínio dono**, não na Sessão — a Sessão só
   despacha se o handler precisar saber qual projeto está ativo.
3. **Não implemente fila, dedup ou "está processando?" do lado Rust
   manualmente para este Command específico** — isso já é comportamento do
   núcleo como um todo (processamento FIFO da fila de Commands), não algo que
   cada handler reimplementa.
4. **A resposta do Command é a confirmação de estado.** O Angular não aplica
   a mudança antes da resposta chegar — sem atualização otimista. O único
   feedback imediato no clique é o controle ficando desabilitado até a
   resposta *daquele* Command chegar (evita duplo clique) — isso é UI
   refletindo "em andamento", não lógica de fila.
5. Se o Command mexe em estado do projeto que precisa sobreviver a um
   restart, confira o skill `atomic-persistence` — não escreva em disco
   direto do handler.
6. Se o Command mexe no `Motor de Áudio` ou toca o callback do `cpal`, pare
   e carregue o skill `realtime-audio-safety` antes de codar.

## Passo a passo: adicionar um Event

1. Cheque a [tabela de eventos](../../../docs/doc.md#tabela-de-eventos) —
   confirme que não existe já um evento que cobre o mesmo dado.
2. **Separe dado estático de dado que muda a cada tick.** É a mesma lição
   que gerou `waveform_ready` separado de `playback_progress`: um payload
   calculado uma vez (waveform, partitura interpretada) não deve ser
   redespachado a cada evento de progresso — isso é IPC redundante. Se o seu
   Event novo parece "a mesma coisa de novo com um campo a mais", provavelmente
   deveria ser um campo derivado no Angular a partir de um Event que já
   existe (ver `ChordBeatStream` em
   [Estado visual em tempo real](../../../docs/doc.md#estado-visual-em-tempo-real-angular)
   como exemplo: é recalculado no cliente cruzando `positionSec` com dados já
   carregados, não um novo Event por tick).
3. **Erros são Events dedicados**, não um campo de erro dentro de um evento
   de sucesso — siga o padrão `audio_error`/`score_parse_error`: causa +
   mensagem, consumido pela UI como estado de erro localizado no componente
   afetado, nunca travando o resto do app.
4. **Apresentação de erro é sempre modal** — não toast, não banner
   persistente
   ([nota de design](../../../docs/doc.md#nota-de-design-apresentação-de-erros-modal)).
   O modal só cobre apresentação: não pausa nem desfaz nada que já estava
   rodando no núcleo.
5. Documente o Event na tabela de eventos do `doc.md` (produtor, payload
   essencial, consumidor) — é o inventário único de todos os Events, mantenha
   sincronizado.

## Checklist de autocorreção antes de considerar terminado

Releia o diff e responda "não" pra todas estas, ou volte e ajuste:

- [ ] Tem alguma fila, dedup, cache de "última resposta" ou lógica de
      decisão no lado Angular? → mover pro Rust.
- [ ] O `Gerenciador de Sessão` ganhou uma regra de negócio nova (não só
      despacho)? → mover pro módulo de domínio certo.
- [ ] A UI aplica o novo estado antes da resposta do Command confirmar? →
      remover atualização otimista.
- [ ] Existe um Event novo carregando dado que já é deduzível de um Event
      existente + estado já carregado no cliente? → derivar no Angular em vez
      de despachar de novo.
- [ ] Erro está sendo mostrado como toast/banner, ou misturado num evento de
      sucesso? → separar em Event de erro + modal.
- [ ] A tabela de eventos do `doc.md` ficou desatualizada? → atualizar.

Se qualquer resposta for "sim", corrija antes de pedir revisão — não deixe
para o agente `rust-core-reviewer`/`angular-shell-reviewer` pegar o que dá
pra resolver sozinho seguindo este checklist.
