# TODO — Decisões pendentes antes da implementação

Decisões que o [`doc.md`](docs/doc.md) deixou em aberto de propósito: não são bugs de
arquitetura, são escolhas de design/implementação que ainda faltam fixar antes de
começar a codar. Marcar `[x]` conforme forem resolvidas (e refletir a decisão no
`doc.md`, se ela afetar o comportamento documentado).

## Núcleo Rust

- [x] **Modelo de concorrência do estado compartilhado** — resolvido: o modelo **mais simples
  de raciocinar**, não o mais sofisticado — `Arc<Mutex<AppState>>` (ou um `Mutex` por módulo, no
  máximo), sem canais nem actor. Handler de `Command` faz lock, muda o que precisa, solta o
  lock; a thread de tempo real do `cpal` nunca toca esse `Mutex` (só lê os atômicos/ring
  buffers já preparados — regra que já existia). Justificativa: o alvo é uma pessoa clicando
  botões, não um servidor concorrente — contenção de lock nesse volume não é um problema real.
  Só considerar algo mais elaborado se profiling mostrar contenção de fato, não como
  otimização especulativa antes de medir. Ver
  [nota de design](docs/doc.md#nota-de-design-modelo-de-concorrência-do-estado-compartilhado).
- [x] **Crates de áudio** — resolvido, priorizando funcionar em Linux/Windows/macOS/Android
  com o menor atrito possível: `symphonia` pra decodificação (puro Rust, sem dependência nativa
  — decodifica igual nas quatro plataformas), `rtrb` pro buffer entre decodificação e o callback
  de tempo real (SPSC, wait-free — mais estreito e mais alinhado ao caso de uso que `ringbuf`),
  `cpal` pra saída de áudio (já cobre as quatro plataformas, incluindo Android via Oboe). Ponto
  de atenção pra implementação (não muda a escolha): o backend Oboe do `cpal` no Android precisa
  do contexto/JVM que o Tauri mobile expõe — wiring a fazer na inicialização, não lógica de
  domínio. Ver [nota de design](docs/doc.md#nota-de-design-crates-de-áudio-multiplataforma).
- [x] **Comportamento no limite do loop** — resolvido: **crossfade curto** (alguns ms), não
  corte seco. Consequência pro `Motor de Áudio`: o início do loop (a partir de `startSec`)
  precisa estar pronto num buffer próprio *antes* do callback de tempo real chegar no fim do
  loop, já que crossfade mistura o trecho que termina com o que começa — não dá pra decodificar
  isso na hora dentro do callback. Duração exata do crossfade e curva (linear/equal-power) ficam
  como detalhe de implementação. Ver
  [nota de design](docs/doc.md#nota-de-design-crossfade-no-limite-do-loop).
- [x] **Canais dos stems (mono/estéreo)** — resolvido: canal é sempre **estéreo** dentro do
  projeto. Stem mono é upmixado (canal duplicado em L/R) por `Importador de Stems` na
  importação, não a cada playback — ao contrário do sample rate, aqui não há rejeição, porque
  duplicar mono em estéreo não tem ambiguidade de qualidade como o resampling teria. `Motor de
  Áudio` nunca lida com buffer mono. Ver
  [Consistência entre stems de um projeto](docs/doc.md#consistência-entre-stems-de-um-projeto).

## Persistência

- [x] **Escrita atômica** — resolvido: **toda** escrita em disco de `Persistência de Projetos`
  é atômica (write-to-temp-file + rename), sem exceção — não só o `.json` do projeto no
  checkpoint com debounce, mas também o cache de waveform e, futuramente, o `.cho`. Ver
  [nota de design](docs/doc.md#nota-de-design-quando-persistência-de-projetos-grava).
- [x] **Cache da waveform** — resolvido: **cacheada em disco**, junto do projeto (campo
  `stems[].waveformCache` no `.json`, escrito atomicamente igual a tudo mais) — não recalculada
  a cada abertura do projeto. Motivo: performance em dispositivos fracos é requisito do projeto,
  e decodificar o stem inteiro pra recalcular picos a cada abertura é justamente o tipo de custo
  que isso evita. Cache é invalidado se o stem for reimportado/substituído. Ver
  [nota de design](docs/doc.md#nota-de-design-cache-da-waveform-em-disco).

## IPC / Angular

- [x] **Gerenciamento de estado no Angular** — resolvido: **Signals**, nativo do Angular — nada
  de NgRx nem outro pacote externo (npm) além do que o próprio framework já traz, pra minimizar
  superfície de ataque de supply chain (ver [Stack e plataforma](docs/doc.md#stack-e-plataforma)).
  Como `Commands` se conectam aos componentes já estava decidido: fila de `Commands` com
  notificação de sucesso/erro por comando; a UI desabilita só o controle que disparou o
  `Command` até a resposta (evita duplo clique), mas outros controles continuam livres para
  disparar seus próprios `Commands`, que entram na mesma fila (ver
  [nota de design](docs/doc.md#nota-de-design-fila-de-commands-no-lado-angular)).
- [x] **UX de erro** — resolvido: **modal**, não toast nem banner persistente, pra `audio_error`
  e `score_parse_error`. O modal é só sobre apresentação — não pausa nem desfaz nada que já
  estava rodando no núcleo; `score_parse_error` continua sem travar o resto do app (dispensa o
  modal, `Visualização de Acordes/Tablatura` fica em estado de erro, resto da tela segue normal).
  Ver [nota de design](docs/doc.md#nota-de-design-apresentação-de-erros-modal).

## Escopo / setup

- [x] **Versão do Tauri** — resolvido: **Tauri v2**, e a política do projeto é manter Tauri e
  Angular sempre na versão estável mais recente (não uma versão fixada de uma vez só) — reforça
  o mesmo objetivo de minimizar superfície de ataque das outras decisões de Angular (nenhum
  pacote de terceiros, Signals nativo). Ver [Stack e plataforma](docs/doc.md#stack-e-plataforma).
  Consequência que fica valendo: o núcleo Rust não deve depender de detalhe de API de uma versão
  específica do Tauri — a ponte `Commands`/`Events` é tratada como substituível, não como parte
  fixa do design do núcleo (mesma separação que permitiria, no limite, trocar a própria camada
  de apresentação, ex.: por Flutter).
- [x] **Edição do `.cho` na v1** — resolvido: o app vai ler **e** atualizar o `.cho` conforme o
  usuário edita acordes no frontend, mas essa edição não entra na v1 — a v1 é só leitura, igual
  já documentado. A diferença é que "escrita" deixou de ser descartada como fora do escopo da
  arquitetura pra sempre: é feature real planejada pra depois da v1 (ver
  [Estado atual e trabalho futuro](docs/doc.md#estado-atual-e-trabalho-futuro)). Quando entrar,
  o fluxo de escrita do `.cho` provavelmente reaproveita o padrão de escrita atômica
  (write-to-temp-file + rename) já decidido em **Escrita atômica** acima, já que os dois casos
  são "não corromper um arquivo de texto do projeto se o app crashar no meio da escrita".
