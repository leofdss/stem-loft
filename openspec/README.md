# OpenSpec in StemLoft

> **Maintenance note:** this doc has a Brazilian Portuguese sibling at
> [`README.pt-BR.md`](./README.pt-BR.md). Whenever one is updated, update
> the other in the same change.

Planned work and committed behavior are tracked with
[OpenSpec](https://github.com/Fission-AI/OpenSpec), a plain-Markdown,
tool-neutral spec workflow. It replaces the hand-maintained task board
`TODO.md` used to hold.

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

- **One change per task.** The change id keeps the stable task number
  (`task-002-project-persistence` is TASK-002) — never reused, even if a
  task is dropped.
- **`tasks.md` is the acceptance criteria.** A change isn't done until
  every box is checked, including its tests and its review.
- **`board.toml` holds what isn't prose**: `status` (`backlog`,
  `in-progress`, `review`; archived changes are Done), `branch`
  (`<type>/<slug>`, matching the Conventional Commits type),
  `depends_on`, `suitable_for` (`agent`, `both` — an agent drafts, a person
  signs off — or `human`), and `playbooks` (`.agents/skills/<name>/SKILL.md`
  to read first).

## Lifecycle

| Step | Command |
|---|---|
| See what's open | `scripts/openspec.sh list` · `scripts/openspec.sh show <change>` · [`docs/kanban.html`](../docs/kanban.html) |
| Propose new work | the `openspec-propose` playbook, or `scripts/openspec.sh new change task-NNN-<slug>` and fill in the artifacts (both languages) + a `board.toml` entry |
| Start / hand in for review | change `status` in `board.toml` |
| Implement | the `openspec-apply-change` playbook; check boxes in both `tasks.md` files as criteria pass |
| Validate | `scripts/openspec.sh validate --all --strict` (part of `scripts/verify.sh`) |
| Finish | `python3 scripts/openspec_archive.py <change>` — never a bare `openspec archive` (see [Languages](#languages)) |
| Refresh the board | `python3 scripts/openspec_to_kanban.py` |

[`scripts/openspec.sh`](../scripts/openspec.sh) pins the CLI version and
runs it through `npx`, so OpenSpec is never a dependency of `ui/`.

## Languages

**OpenSpec itself supports one language per project**: you set it in
`config.yaml`'s `context` (or `openspec init --language`), and it writes
every artifact in that language. Structural headings (`## Why`,
`### Requirement:`, `#### Scenario:`, ...) and the normative keywords
`SHALL`/`MUST` (and `WHEN`/`THEN`) must stay in English in any language,
because validation relies on them. It has no notion of the same artifact in
two languages, which this project needs.

So, on top of OpenSpec:

- **English is canonical.** The files OpenSpec reads (`proposal.md`,
  `tasks.md`, `design.md`, `specs/**/spec.md`) are in English and are what
  validation and archiving act on.
- **Every artifact has a pt-BR counterpart**, with the same headings and
  keywords in English and the prose in Portuguese:
  - next to the English file as `<name>.pt-BR.md` — proposal, tasks,
    design, and main specs (`openspec/specs/<cap>/spec.pt-BR.md`);
  - for a change's delta specs, under
    `changes/<id>/i18n/pt-BR/specs/<cap>/spec.md`, because OpenSpec's
    validator rejects any extra file inside a change's `specs/` folder.
- [`scripts/openspec_archive.py`](../scripts/openspec_archive.py) archives
  in both languages: it runs the same pinned `openspec archive` on a
  temporary copy where the pt-BR files sit at OpenSpec's paths, copies the
  merged result back to `spec.pt-BR.md`, then archives the real change.
- [`scripts/check_bilingual_parity.py`](../scripts/check_bilingual_parity.py)
  fails when an OpenSpec artifact has no pt-BR counterpart or when the two
  drift structurally.
