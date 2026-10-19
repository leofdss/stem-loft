# TODO — Board de tarefas de implementação

> **Nota de manutenção:** este documento tem um irmão em inglês em
> [`TODO.md`](./TODO.md). Sempre que um for atualizado, atualize o outro
> na mesma mudança — não deixe os dois divergirem.

Todas as decisões de pré-implementação (modelo de concorrência, crates de
áudio, escritas atômicas, gerenciamento de estado no Angular, UX de erros,
versão do Tauri, escopo de edição do `.cho`) estão **resolvidas** e
documentadas como notas de design em
[`docs/architecture.md`](docs/architecture.md).

## Onde o board fica

O board de tarefas da v1 foi para o [OpenSpec](openspec/README.pt-BR.md):
cada tarefa é uma change em `openspec/changes/task-NNN-<slug>/` — a
proposta, os critérios de aceite (`tasks.md`) e o comportamento que ela
adiciona — com status, branch, dependências e playbooks no
[`openspec/board.toml`](openspec/board.toml). Tarefas concluídas são
arquivadas em `openspec/changes/archive/`, e o comportamento que
adicionaram passa a fazer parte de `openspec/specs/`.

- Board visual: [`docs/kanban.html`](docs/kanban.html), gerado pelo
  [`scripts/openspec_to_kanban.py`](scripts/openspec_to_kanban.py).
- Pelo terminal: `scripts/openspec.sh list`, depois
  `scripts/openspec.sh show <change>`.
- Como propor, começar, concluir e arquivar uma tarefa (nos dois idiomas):
  [`openspec/README.pt-BR.md`](openspec/README.pt-BR.md#ciclo-de-vida).

## Icebox (pós-v1, ainda não quebrado em tarefas)

Fora do escopo do board acima, mas já reconhecido em
[Status atual e trabalho futuro](docs/architecture.md#current-status-and-future-work)
— não comece nada disso sem antes transformar em sua própria change do
OpenSpec (`task-NNN-<slug>`) com critérios de aceite:

- Separação automática de stems (`Separation Client` no núcleo,
  `Automatic Separation` na UI, o serviço externo `Stem Separation API`) —
  precisa de um padrão assíncrono de job (polling ou webhook) ainda não
  projetado.
- Editar acordes/tablatura no `Chord/Tab View`, com o núcleo reescrevendo o
  `.cho` (reaproveitando o padrão de escrita atômica da TASK-002).
