# TODO — Decisões pendentes antes da implementação

Decisões que o [`doc.md`](docs/doc.md) deixou em aberto de propósito: não são bugs de
arquitetura, são escolhas de design/implementação que ainda faltam fixar antes de
começar a codar. Marcar `[x]` conforme forem resolvidas (e refletir a decisão no
`doc.md`, se ela afetar o comportamento documentado).

## Núcleo Rust

- [ ] **Modelo de concorrência do estado compartilhado** — como `SESSION`/`LOOPMGR`/`PERSIST`
  são acessados por handlers de `Command` (async, potencialmente concorrentes) e pela
  thread de callback do `cpal` ao mesmo tempo. `Arc<Mutex<T>>` por módulo? Um único
  estado protegido? Actor/canal? Molda a assinatura de praticamente todo módulo — é a
  primeira coisa a fixar.
- [ ] **Crates de áudio** — decodificador (`symphonia` é o candidato natural), buffer
  entre decodificação e o callback real-time (`ringbuf`/`rtrb`), e como negociar sample
  rate/formato com o dispositivo via `cpal`.
- [ ] **Comportamento no limite do loop** — corte seco na volta pro marcador inicial
  pode soar como clique se não cair num zero-crossing. Decidir: corte seco (mais
  simples) ou crossfade curto (alguns ms) — afeta o design do `Motor de Áudio` desde o
  início.
- [ ] **Canais dos stems (mono/estéreo)** — a regra de sample rate já existe (rejeita
  divergência na importação); mono vs. estéreo ainda não tem regra. Aceitar só um
  formato, ou fazer up/downmix?

## Persistência

- [ ] **Escrita atômica** — o debounce já está definido no `doc.md`, mas não *como* se
  escreve. Write-to-temp-file + rename evita corromper o `.json` se o app crashar no
  meio de uma escrita.
- [ ] **Cache da waveform** — os picos de `waveform_ready` são recalculados toda vez
  que o projeto abre (em memória), ou ficam cacheados em disco junto do projeto? Se
  cacheado, vira mais um campo no schema do `.json`.

## IPC / Angular

- [ ] **Gerenciamento de estado no Angular** — services + RxJS puro, Signals, ou NgRx.
  Decide como `Commands`/`Events` se conectam aos componentes (`Mixer`, `Timeline`,
  `ChordView`).
- [ ] **UX de erro** — `audio_error` e `score_parse_error` já existem como eventos, mas
  não como aparecem na tela: toast, banner persistente, modal bloqueante? Importa
  principalmente pro `score_parse_error`, que não deve travar o resto do app.

## Escopo / setup

- [ ] **Versão do Tauri** (v1 vs v2 — a API de `Commands`/`Events` mudou entre elas) e
  versão do Angular — nenhuma das duas está fixada no `doc.md`.
- [ ] **Edição do `.cho` na v1** — o `doc.md` já decidiu que o app só *lê* o `.cho`
  (autoria é externa); confirmar que isso vale pro MVP inteiro, senão "Visualização de
  Acordes/Tablatura" ganha escopo de editor sem estar nos componentes documentados.
