# OpenSpec no StemLoft

> **Nota de manutenção:** este documento tem um irmão em inglês em
> [`README.md`](./README.md). Sempre que um for atualizado, atualize o
> outro na mesma mudança.

O trabalho planejado e o comportamento já assumido pelo app são
acompanhados com o [OpenSpec](https://github.com/Fission-AI/OpenSpec), um
fluxo de specs em Markdown puro, que não depende de ferramenta. Ele
substitui o board de tarefas mantido à mão que ficava no `TODO.md`.

## Layout

```text
openspec/
  config.yaml                 # OpenSpec project config
  board.toml                  # status, branch, dependencies, playbooks per active change
  specs/<capability>/
    spec.md                   # behavior the app already commits to (EN)
    spec.pt-BR.md             # same, pt-BR
  changes/task-NNN-<slug>/    # one per task, still open
    .openspec.yaml
    proposal.md / proposal.pt-BR.md   # why, what changes, capabilities, impact
    tasks.md    / tasks.pt-BR.md      # acceptance criteria as checkboxes
    specs/<capability>/spec.md        # behavior this change adds (EN)
    i18n/pt-BR/specs/<capability>/spec.md   # same, pt-BR
  changes/archive/<date>-task-NNN-<slug>/   # finished tasks
```

- **Uma change por tarefa.** O id da change mantém o número estável da
  tarefa (`task-002-project-persistence` é a TASK-002) — nunca reutilizado,
  mesmo que a tarefa seja descartada.
- **O `tasks.md` são os critérios de aceite.** Uma change só está pronta
  quando todas as caixas estão marcadas, incluindo os testes e a revisão.
- **O `board.toml` guarda o que não é prosa**: `status` (`backlog`,
  `in-progress`, `review`; changes arquivadas estão Done), `branch`
  (`<tipo>/<slug>`, com o mesmo tipo do Conventional Commits),
  `depends_on`, `suitable_for` (`agent`, `both` — um agente faz o
  rascunho, uma pessoa aprova — ou `human`) e `playbooks`
  (`.agents/skills/<nome>/SKILL.md` para ler antes).

## Ciclo de vida

| Etapa | Comando |
|---|---|
| Ver o que está aberto | `scripts/openspec.sh list` · `scripts/openspec.sh show <change>` · [`docs/kanban.html`](../docs/kanban.html) |
| Propor trabalho novo | o playbook `openspec-propose`, ou `scripts/openspec.sh new change task-NNN-<slug>` e preencher os artefatos (nos dois idiomas) + uma entrada no `board.toml` |
| Começar / entregar para revisão | mudar o `status` no `board.toml` |
| Implementar | o playbook `openspec-apply-change`; marcar as caixas nos dois `tasks.md` conforme os critérios passam |
| Validar | `scripts/openspec.sh validate --all --strict` (faz parte do `scripts/verify.sh`) |
| Concluir | `python3 scripts/openspec_archive.py <change>` — nunca um `openspec archive` direto (veja [Idiomas](#idiomas)) |
| Atualizar o board | `python3 scripts/openspec_to_kanban.py` |

O [`scripts/openspec.sh`](../scripts/openspec.sh) fixa a versão da CLI e a
roda via `npx`, então o OpenSpec nunca vira dependência do `ui/`.

## Idiomas

**O OpenSpec em si suporta um idioma por projeto**: você o define no
`context` do `config.yaml` (ou com `openspec init --language`), e ele
escreve todos os artefatos nesse idioma. Os títulos estruturais
(`## Why`, `### Requirement:`, `#### Scenario:`, ...) e as palavras-chave
normativas `SHALL`/`MUST` (e `WHEN`/`THEN`) precisam ficar em inglês em
qualquer idioma, porque a validação depende delas. Ele não tem a noção de
um mesmo artefato em dois idiomas, que é o que este projeto precisa.

Por isso, por cima do OpenSpec:

- **O inglês é o canônico.** Os arquivos que o OpenSpec lê (`proposal.md`,
  `tasks.md`, `design.md`, `specs/**/spec.md`) ficam em inglês e são eles
  que a validação e o arquivamento usam.
- **Todo artefato tem uma versão pt-BR**, com os mesmos títulos e
  palavras-chave em inglês e a prosa em português:
  - ao lado do arquivo em inglês, como `<nome>.pt-BR.md` — proposal,
    tasks, design e specs principais (`openspec/specs/<cap>/spec.pt-BR.md`);
  - para as specs delta de uma change, em
    `changes/<id>/i18n/pt-BR/specs/<cap>/spec.md`, porque o validador do
    OpenSpec rejeita qualquer arquivo extra dentro da pasta `specs/` de
    uma change.
- O [`scripts/openspec_archive.py`](../scripts/openspec_archive.py)
  arquiva nos dois idiomas: roda o mesmo `openspec archive` fixado numa
  cópia temporária onde os arquivos pt-BR estão nos caminhos que o
  OpenSpec espera, copia o resultado mesclado de volta para o
  `spec.pt-BR.md` e então arquiva a change real.
- O [`scripts/check_bilingual_parity.py`](../scripts/check_bilingual_parity.py)
  falha quando um artefato do OpenSpec não tem versão pt-BR ou quando as
  duas divergem na estrutura.
