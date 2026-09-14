# TODO — Board de tarefas de implementação

> **Nota de manutenção:** este documento tem uma versão em inglês em
> [`TODO.md`](./TODO.md). Sempre que um dos dois for atualizado,
> atualize o outro na mesma alteração — não deixe as duas versões divergirem.

Todas as decisões pré-implementação que este documento rastreava (modelo de
concorrência, crates de áudio, escritas atômicas, gerenciamento de estado no
Angular, UX de erro, versão do Tauri, escopo de edição do `.cho`) estão
**resolvidas** e documentadas como notas de design em
[`docs/architecture.md`](docs/architecture.md) — veja o histórico do git se
precisar da discussão original. Este documento agora é o **board de
implementação da v1**: tarefas concretas, dimensionadas para uma única
sessão de um humano ou de um agente de IA, cada uma com critérios de aceite
e a branch que a implementa.

## Como usar este board

- As colunas são `## Backlog` → `## In Progress` → `## Review` → `## Done`
  (nomes das colunas mantidos em inglês de propósito, para que
  `scripts/todo_to_kanban.py` funcione igual nos dois arquivos). Mova uma
  tarefa cortando seu bloco `### TASK-NNN` para a coluna de destino.
- Cada tarefa tem um id estável `TASK-NNN` (nunca reaproveitado, mesmo se a
  tarefa for descartada), uma **Área**, para quem ela é realisticamente
  **indicada**, a(s) **skill(s)** da tabela no [`CLAUDE.md`](CLAUDE.md) raiz
  a carregar primeiro, a **branch** que a implementa, e do que ela
  **depende**.
- Nomes de branch seguem `<tipo>/<slug>`, espelhando o tipo do [Conventional
  Commits](https://www.conventionalcommits.org/) do(s) commit(s) que chegam
  nela (majoritariamente `feat/`; `test/` e `chore/` aparecem em tarefas que
  não são feature) — imposto para mensagens de commit pelo `bash_guard.py`,
  não para nomes de branch, mas mantenha os dois alinhados.
- "Indicada para: Agente de IA" significa que a tarefa é autocontida o
  suficiente para entregar a um agente com as skills/hooks deste repositório
  carregados. "Humano" marca tarefas que exigem um julgamento que um hook
  não consegue checar (isso soa certo em um fone de ouvido de verdade, isso
  parece certo na mão) — um agente de IA ainda pode fazer uma primeira
  versão, mas um humano precisa aprovar antes de `Done`.
- **Todo agente de IA que mudar a estrutura de tarefas/colunas deste
  arquivo (ou do `TODO.md`) precisa regenerar
  [`docs/kanban.html`](docs/kanban.html) antes de considerar a mudança
  concluída:**

  ```bash
  python3 scripts/todo_to_kanban.py --html
  ```

  Isso é reforçado pelo hook `kanban_sync_reminder.py`, mas não confie em
  notar o lembrete — rode o comando como parte da própria edição. O comando
  lê tanto o `TODO.md` quanto o `TODO.pt-BR.md` e escreve a página inteira
  de forma determinística; é uma função pura desses dois arquivos, então o
  resultado nunca depende de quem roda o comando ou de como interpretou
  este documento — nunca edite `docs/kanban.html` diretamente. Também dá
  para obter só o bloco Mermaid puro (por exemplo, para colar em outro
  lugar) com:

  ```bash
  python3 scripts/todo_to_kanban.py TODO.pt-BR.md            # imprime no stdout
  python3 scripts/todo_to_kanban.py TODO.pt-BR.md -o docs/kanban.pt-BR.md
  ```

  Veja [`scripts/todo_to_kanban.py`](scripts/todo_to_kanban.py) para como
  ele interpreta este arquivo — é guiado inteiramente pela estrutura de
  `##`/`###` abaixo, então mantenha novas tarefas nesse formato.

## Backlog

### TASK-001 — Session/State Manager e fiação do AppState
- **Área:** Núcleo Rust — Session/State Manager
- **Indicada para:** Agente de IA
- **Skill(s):** `rust-domain-module`, `add-ipc-contract`
- **Branch:** `feat/session-app-state`
- **Depende de:** —

Substituir o stub `session.rs` por um `AppState` real atrás de um único
`Arc<Mutex<AppState>>` (ou no máximo um `Mutex` por módulo), e conectar os
construtores de injeção manual dos módulos de domínio para os quais o
Session roteia `Commands`.

**Critérios de aceite:**
- [ ] `AppState` corresponde à [nota de design do modelo de concorrência](docs/architecture.md#design-note-concurrency-model-for-shared-state) — sem channels, sem actor
- [ ] `Session` guarda apenas "qual projeto está aberto"; todo handler de `Command` delega ao módulo dono daquele domínio, conforme [Session as a thin router](docs/architecture.md#design-note-session-as-a-thin-router)
- [ ] `cargo check`/`cargo clippy` limpos (via `remote-build-offload`)
- [ ] Revisado pelo `rust-core-reviewer`

### TASK-002 — Project Persistence: carregar/salvar, escritas atômicas, schemaVersion
- **Área:** Núcleo Rust — Project Persistence
- **Indicada para:** Agente de IA
- **Skill(s):** `atomic-persistence`
- **Branch:** `feat/project-persistence`
- **Depende de:** TASK-001

Implementar `persistence.rs`: ler/escrever o `.json` do projeto conforme o
[schema](docs/architecture.md#project-versioning), escrita em arquivo
temporário + rename sem exceções, e os três casos de `schemaVersion` (mesma
versão carrega direto, versão menor roda migrações, versão maior gera erro
explícito).

**Critérios de aceite:**
- [ ] Toda escrita passa por arquivo temporário + `rename`, nunca uma escrita direta no caminho final
- [ ] Escritas acontecem em um checkpoint com debounce, não a cada `Command` (veja [nota de design](docs/architecture.md#design-note-when-project-persistence-writes))
- [ ] Os três casos de `schemaVersion` têm teste cobrindo
- [ ] Um crash simulado no meio de uma escrita deixa o arquivo anterior intacto (teste)
- [ ] Revisado pelo `rust-core-reviewer`

### TASK-003 — Stem Importer: importação manual, validação, upmix de mono
- **Área:** Núcleo Rust — Stem Importer
- **Indicada para:** Agente de IA
- **Skill(s):** `rust-domain-module`
- **Branch:** `feat/stem-importer`
- **Depende de:** TASK-001, TASK-002

Implementar a importação manual de stems: checagem de consistência de taxa
de amostragem, upmix de mono para estéreo, e extração de `durationSec` do
cabeçalho do arquivo (sem decodificação completa).

**Critérios de aceite:**
- [ ] Um stem com taxa de amostragem diferente dos já importados é rejeitado com erro explícito, não resampleado silenciosamente
- [ ] Um stem mono é convertido para estéreo na importação — `Audio Engine` nunca vê um buffer mono
- [ ] `durationSec` é lido do cabeçalho e armazenado conforme [Consistência entre os stems de um projeto](docs/architecture.md#consistency-across-a-projects-stems)
- [ ] Revisado pelo `rust-core-reviewer`

### TASK-004 — Loop & Marker Manager
- **Área:** Núcleo Rust — Loop & Marker Manager
- **Indicada para:** Agente de IA
- **Skill(s):** `rust-domain-module`
- **Branch:** `feat/loop-marker-manager`
- **Depende de:** TASK-003

`set_markers`/estado da seção em loop, validado contra a duração do
projeto.

**Critérios de aceite:**
- [ ] `endSec` além da duração do projeto (o `durationSec` do stem mais longo) é rejeitado, não truncado silenciosamente
- [ ] Os marcadores atuais são expostos ao `Audio Engine` para a reprodução em loop
- [ ] Revisado pelo `rust-core-reviewer`

### TASK-005 — Audio Engine: saída cpal + decodificação symphonia + pipeline rtrb
- **Área:** Núcleo Rust — Audio Engine
- **Indicada para:** Ambos — agente de IA faz a primeira versão, humano valida em um dispositivo real
- **Skill(s):** `realtime-audio-safety`
- **Branch:** `feat/audio-engine-playback`
- **Depende de:** TASK-001, TASK-003

Conectar decodificação `symphonia` → ring buffer `rtrb` → callback `cpal`
para reprodução básica (ainda sem loop/crossfade — isso é a TASK-006).

**Critérios de aceite:**
- [ ] Decodificação e qualquer I/O acontecem fora do callback do `cpal`; o callback só lê buffers já preparados
- [ ] Nenhuma alocação, lock ou I/O dentro do callback (checado contra a checklist de `realtime-audio-safety`)
- [ ] Um humano confirma reprodução audível e sem glitches em pelo menos um dispositivo real
- [ ] Revisado pelo `rust-core-reviewer`

### TASK-006 — Crossfade na fronteira do loop
- **Área:** Núcleo Rust — Audio Engine
- **Indicada para:** Ambos — agente de IA faz a primeira versão, humano julga a qualidade do áudio
- **Skill(s):** `realtime-audio-safety`
- **Branch:** `feat/audio-engine-crossfade`
- **Depende de:** TASK-004, TASK-005

Crossfade curto (alguns ms) na fronteira do loop em vez de um corte seco,
conforme [a nota de design](docs/architecture.md#design-note-crossfade-at-the-loop-boundary).

**Critérios de aceite:**
- [ ] O buffer do início do loop está pronto antes de o callback chegar em `endSec`
- [ ] O crossfade é matemática pura sobre amostras já decodificadas dentro do callback — sem nova alocação/I/O
- [ ] Um humano escuta a fronteira do loop em pelo menos um projeto real e confirma ausência de cliques/artefatos

### TASK-007 — Cálculo de picos de waveform + cache em disco
- **Área:** Núcleo Rust — Audio Engine / Project Persistence
- **Indicada para:** Agente de IA
- **Skill(s):** `atomic-persistence`, `realtime-audio-safety`
- **Branch:** `feat/waveform-cache`
- **Depende de:** TASK-002, TASK-003

**Critérios de aceite:**
- [ ] Picos são calculados uma vez (na importação, ou na primeira abertura se não houver cache) e escritos pelo caminho de escrita atômica
- [ ] O cache é invalidado quando o stem correspondente é reimportado/substituído
- [ ] `waveform_ready` serve os picos do cache em disco quando presente, sem redecodificar

### TASK-008 — Estado do mixer (volume/mute/solo), propagação segura para tempo real
- **Área:** Núcleo Rust — Audio Engine
- **Indicada para:** Agente de IA
- **Skill(s):** `realtime-audio-safety`
- **Branch:** `feat/mixer-state`
- **Depende de:** TASK-005

**Critérios de aceite:**
- [ ] Uma mudança de volume/mute/solo escreve em um atomic/double-buffer que o callback lê — nunca um `Mutex` no qual ele possa bloquear
- [ ] Uma mudança no mixer marca o projeto como sujo para o checkpoint com debounce (TASK-002), não uma escrita a cada tick

### TASK-009 — Superfície de IPC Commands
- **Área:** Communication Bridge — Tauri IPC
- **Indicada para:** Agente de IA
- **Skill(s):** `add-ipc-contract`
- **Branch:** `feat/ipc-commands`
- **Depende de:** TASK-001

**Critérios de aceite:**
- [ ] A fila de `Commands` mora inteiramente no núcleo Rust; nada no Angular enfileira, deduplica ou reordena
- [ ] Cada handler de `Command` apenas despacha para o módulo dono daquele domínio — sem lógica de coordenação no próprio handler
- [ ] Revisado pelo `rust-core-reviewer` e pelo `angular-shell-reviewer` (pontos de chamada)

### TASK-010 — Superfície de IPC Events (playback_progress, waveform_ready, transport_state_changed, audio_error)
- **Área:** Communication Bridge — Tauri IPC
- **Indicada para:** Agente de IA
- **Skill(s):** `add-ipc-contract`
- **Branch:** `feat/ipc-events`
- **Depende de:** TASK-005, TASK-007

**Critérios de aceite:**
- [ ] Cada evento corresponde exatamente à [tabela de eventos](docs/architecture.md#events-table) (produtor, payload, consumidores)
- [ ] `audio_error` é de fato alcançável em teste manual (ex.: desconectar o dispositivo de saída)

### TASK-011 — Parser do `.cho` (Score Metadata Manager)
- **Área:** Núcleo Rust — Score Metadata Manager
- **Indicada para:** Agente de IA
- **Skill(s):** `chordpro-format`
- **Branch:** `feat/cho-parser`
- **Depende de:** —

**Critérios de aceite:**
- [ ] Toda regra de gramática da skill `chordpro-format` (um acorde por linha, âncoras `{t:}`, gramática de tablatura, resolução de `startSec`/`endSec`) tem um teste unitário passando
- [ ] Uma diretiva desconhecida é ignorada, não fatal
- [ ] `score_parse_error` carrega a linha e a mensagem do problema sem travar o resto do app
- [ ] Revisado pelo `rust-core-reviewer`

### TASK-012 — Evento `score_loaded` + fatia de acorde/batida em tempo real
- **Área:** Núcleo Rust / IPC — Score Metadata Manager
- **Indicada para:** Agente de IA
- **Skill(s):** `add-ipc-contract`, `chordpro-format`
- **Branch:** `feat/score-events`
- **Depende de:** TASK-011

**Critérios de aceite:**
- [ ] `score_loaded` dispara uma vez por parse com `tempo`/`time`/`tuning` do cabeçalho
- [ ] O payload em tempo real a cada tick carrega só o acorde/batida ativo, não a partitura inteira (veja [Real-time visual state](docs/architecture.md#real-time-visual-state-angular))

### TASK-013 — Tela de Importação de Stems (Angular)
- **Área:** Angular — Stem Import Screen
- **Indicada para:** Ambos
- **Skill(s):** `add-ipc-contract`
- **Branch:** `feat/ui-stem-import-screen`
- **Depende de:** TASK-003, TASK-009

**Critérios de aceite:**
- [ ] Dispara o `Command` de importação imediatamente, sem fila própria no cliente ou atualização otimista
- [ ] O controle que disparou fica desabilitado até a resposta daquele `Command` chegar
- [ ] Revisado pelo `angular-shell-reviewer`

### TASK-014 — Controles de Transporte (Angular)
- **Área:** Angular — Transport Controls
- **Indicada para:** Ambos
- **Skill(s):** `add-ipc-contract`
- **Branch:** `feat/ui-transport-controls`
- **Depende de:** TASK-009, TASK-010

**Critérios de aceite:**
- [ ] Reflete `transport_state_changed`; nunca assume um estado antes de o evento chegar
- [ ] Revisado pelo `angular-shell-reviewer`

### TASK-015 — Timeline + Waveform (Angular)
- **Área:** Angular — Timeline + Waveform
- **Indicada para:** Ambos
- **Skill(s):** `add-ipc-contract`
- **Branch:** `feat/ui-timeline-waveform`
- **Depende de:** TASK-007, TASK-009, TASK-010

**Critérios de aceite:**
- [ ] Renderiza os picos de `waveform_ready` e a posição de `playback_progress`
- [ ] A seleção de marcador/loop envia `set_markers` e espera a resposta antes de mover o marcador na tela
- [ ] Revisado pelo `angular-shell-reviewer`

### TASK-016 — Mixer de Stems (Angular)
- **Área:** Angular — Stem Mixer
- **Indicada para:** Ambos
- **Skill(s):** `add-ipc-contract`
- **Branch:** `feat/ui-stem-mixer`
- **Depende de:** TASK-008, TASK-009

**Critérios de aceite:**
- [ ] Arrastar um fader não dispara uma escrita de persistência a cada tick (verificado contra o debounce da TASK-002/TASK-008)
- [ ] Revisado pelo `angular-shell-reviewer`

### TASK-017 — Chord/Tab View (Angular, somente leitura)
- **Área:** Angular — Chord/Tab View
- **Indicada para:** Ambos
- **Skill(s):** `add-ipc-contract`, `chordpro-format`
- **Branch:** `feat/ui-chord-tab-view`
- **Depende de:** TASK-012, TASK-010

**Critérios de aceite:**
- [ ] Renderiza acordes/tablatura/letra sincronizados com a posição de reprodução
- [ ] `score_parse_error` coloca só essa view em estado de erro — o resto da tela continua funcionando
- [ ] Revisado pelo `angular-shell-reviewer`

### TASK-018 — Modal de Erro (Angular)
- **Área:** Angular — Error Modal
- **Indicada para:** Ambos
- **Skill(s):** `add-ipc-contract`
- **Branch:** `feat/ui-error-modal`
- **Depende de:** TASK-010, TASK-012

**Critérios de aceite:**
- [ ] `audio_error` e `score_parse_error` abrem o modal com a causa e a mensagem do evento
- [ ] Fechar o modal não pausa nem desfaz nada que já esteja rodando no núcleo
- [ ] Revisado pelo `angular-shell-reviewer`

### TASK-019 — Passe manual de QA ponta a ponta (golden path da v1)
- **Área:** App inteiro
- **Indicada para:** Humano
- **Skill(s):** `run`
- **Branch:** `test/v1-golden-path`
- **Depende de:** TASK-001 – TASK-018

**Critérios de aceite:**
- [ ] Importar stems → marcar loop → reproduzir com crossfade → ajustar mixer → fechar e reabrir o projeto restaura o estado → `.cho` mostra acordes sincronizados com a reprodução, tudo em um dispositivo real
- [ ] Todo bug encontrado é registrado como uma nova tarefa em `Backlog` aqui, não corrigido de forma improvisada dentro desta tarefa

## In Progress

## Review

## Done

## Icebox (pós-v1, ainda não quebrado em tarefas)

Fora do escopo do board acima, mas já reconhecido em
[Status atual e trabalho futuro](docs/architecture.md#current-status-and-future-work)
— não comece nada disso sem antes transformar em suas próprias entradas
`TASK-NNN` com critérios de aceite:

- Separação automática de stems (`Separation Client` no núcleo,
  `Automatic Separation` na UI, o serviço externo `Stem Separation API`) —
  precisa de um padrão assíncrono de job (polling ou webhook) ainda não
  projetado.
- Editar acordes/tablatura no `Chord/Tab View`, com o núcleo reescrevendo o
  `.cho` (reaproveitando o padrão de escrita atômica da TASK-002).
