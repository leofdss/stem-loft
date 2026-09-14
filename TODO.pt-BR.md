# TODO — Decisões pendentes antes da implementação

> **Nota de manutenção:** este documento tem uma versão em inglês em
> [`TODO.md`](./TODO.md). Sempre que um dos dois for atualizado,
> atualize o outro na mesma alteração — não deixe as duas versões divergirem.

Decisões que o [`architecture.md`](docs/architecture.md) deliberadamente deixou em aberto:
essas não são bugs de arquitetura, são escolhas de design/implementação que ainda
precisam ser fechadas antes de o código começar. Marque com `[x]` conforme forem
resolvidas (e reflita a decisão em `architecture.md` se ela afetar comportamento
documentado).

## Núcleo em Rust

- [x] **Modelo de concorrência para estado compartilhado** — resolvido: o modelo
  **mais simples de raciocinar**, não o mais sofisticado — `Arc<Mutex<AppState>>` (ou
  no máximo um `Mutex` por módulo), sem channels, sem actor. Um handler de `Command`
  trava, altera o que precisa, libera o lock; a thread de tempo real do `cpal` nunca
  toca esse `Mutex` (ela só lê os atomics/ring buffers já preparados — uma regra que
  já existia). Justificativa: o alvo é uma pessoa clicando botões, não um servidor
  concorrente — contenção de lock nesse volume não é um problema real. Só considerar
  algo mais elaborado se o profiling mostrar contenção de fato, não como otimização
  especulativa. Veja a
  [nota de design](docs/architecture.md#design-note-concurrency-model-for-shared-state).
- [x] **Crates de áudio** — resolvido, priorizando rodar em Linux/Windows/macOS/Android
  com o mínimo de atrito possível: `symphonia` para decodificação (Rust puro, sem
  dependência nativa — decodifica de forma idêntica nas quatro plataformas), `rtrb`
  para o buffer entre a decodificação e o callback de tempo real (SPSC, wait-free —
  mais estreito e mais alinhado ao caso de uso do que `ringbuf`), `cpal` para saída de
  áudio (já cobre as quatro plataformas, incluindo Android via Oboe). Ponto de atenção
  de implementação (não muda a escolha): o backend Oboe do `cpal` no Android precisa
  do contexto/JVM que o Tauri mobile expõe — fiação a fazer na inicialização, não
  lógica de domínio. Veja a
  [nota de design](docs/architecture.md#design-note-audio-crates-cross-platform).
- [x] **Comportamento na fronteira do loop** — resolvido: um **crossfade curto**
  (alguns ms), não um corte seco. Consequência para o `Audio Engine`: o início do loop
  (a partir de `startSec`) precisa estar pronto em seu próprio buffer *antes* de o
  callback de tempo real chegar ao fim do loop, já que o crossfade mistura o trecho
  que termina com o que começa — não há decodificação disso em tempo real dentro do
  callback. A duração exata do crossfade e a curva (linear/potência igual) continuam
  sendo detalhe de implementação. Veja a
  [nota de design](docs/architecture.md#design-note-crossfade-at-the-loop-boundary).
- [x] **Canais dos stems (mono/estéreo)** — resolvido: os canais são sempre
  **estéreo** dentro do projeto. Um stem mono é convertido para estéreo (canal
  duplicado para L/R) pelo `Stem Importer` no momento da importação, não a cada
  reprodução — ao contrário da taxa de amostragem, não há rejeição aqui, porque
  duplicar mono em estéreo não tem a ambiguidade de qualidade que o resampling teria.
  O `Audio Engine` nunca lida com buffers mono. Veja
  [Consistência entre os stems de um projeto](docs/architecture.md#consistency-across-a-projects-stems).

## Persistência

- [x] **Escritas atômicas** — resolvido: **toda** escrita em disco feita pelo
  `Project Persistence` é atômica (escreve em arquivo temporário + rename), sem
  exceções — não só o `.json` do projeto no checkpoint com debounce, mas também o
  cache de waveform e, no futuro, o `.cho`. Veja a
  [nota de design](docs/architecture.md#design-note-when-project-persistence-writes).
- [x] **Cache de waveform** — resolvido: **cacheado em disco**, junto com o projeto
  (o campo `stems[].waveformCache` no `.json`, escrito atomicamente como tudo o mais)
  — não recalculado toda vez que o projeto abre. Motivo: desempenho em dispositivos
  fracos é um requisito do projeto, e decodificar o stem inteiro para recalcular picos
  a cada abertura é exatamente o tipo de custo que isso evita. O cache é invalidado se
  o stem for reimportado/substituído. Veja a
  [nota de design](docs/architecture.md#design-note-waveform-cache-on-disk).

## IPC / Angular

- [x] **Gerenciamento de estado no Angular** — resolvido: **Signals**, nativo do
  Angular — sem NgRx, sem nenhum outro pacote externo (npm) além do que o próprio
  framework já fornece, para minimizar a superfície de ataque da cadeia de
  suprimentos (veja [Stack e plataforma](docs/architecture.md#stack-and-platform)).
  Como os `Commands` se conectam aos componentes já havia sido decidido, com uma
  correção: a fila de `Commands` mora no **núcleo em Rust**, não no Angular — o
  Angular só apresenta o estado que o Rust reporta e envia a intenção do usuário, sem
  fila/lógica própria (o mesmo raciocínio de "toda a inteligência mora no Rust" usado
  para minimizar o custo de uma futura troca de framework). O efeito para o usuário
  continua o mesmo: o controle que disparou um `Command` fica desabilitado até a
  resposta (evitando duplo clique), outros controles continuam livres para disparar
  seus próprios `Commands`, que o núcleo processa em sequência. Veja a
  [nota de design](docs/architecture.md#design-note-commands-queue-in-the-rust-core).
- [x] **UX de erro** — resolvido: um **modal**, não um toast ou banner persistente,
  para `audio_error` e `score_parse_error`. O modal é só sobre apresentação — ele não
  pausa nem desfaz nada que já esteja rodando no núcleo; `score_parse_error` continua
  não travando o resto do app (fecha o modal, o `Chord/Tab View` fica em estado de
  erro, o resto da tela continua funcionando normalmente).
  Veja a [nota de design](docs/architecture.md#design-note-error-presentation-modal).

## Escopo / setup

- [x] **Versão do Tauri** — resolvido: **Tauri v2**, e a política do projeto é
  manter Tauri e Angular sempre na última versão estável (não fixado uma vez e
  esquecido) — reforçando o mesmo objetivo de minimização de superfície de ataque das
  outras decisões do Angular (sem pacotes de terceiros, Signals nativo). Veja
  [Stack e plataforma](docs/architecture.md#stack-and-platform).
  Consequência que continua valendo: o núcleo em Rust não pode depender de detalhes
  de API de uma versão específica do Tauri — a ponte de `Commands`/`Events` é tratada
  como substituível, não como parte fixa do design do núcleo (a mesma separação que,
  no limite, permitiria trocar a própria camada de apresentação, por exemplo por
  Flutter).
- [x] **Editar o `.cho` na v1** — resolvido: o app vai tanto ler **quanto** atualizar
  o `.cho` conforme o usuário edita acordes no frontend, mas essa edição não faz parte
  da v1 — a v1 é somente leitura, como já documentado. A diferença é que "escrever"
  deixou de ser descartado como para sempre fora do escopo da arquitetura: é uma
  funcionalidade real planejada para depois da v1 (veja
  [Status atual e trabalho futuro](docs/architecture.md#current-status-and-future-work)). Quando
  chegar, o fluxo de escrita do `.cho` provavelmente reaproveitará o padrão de escrita
  atômica (escreve em arquivo temporário + rename) já decidido em **Escritas
  atômicas** acima, já que os dois casos se resumem a "não corromper um arquivo de
  texto do projeto se o app travar no meio da escrita".
