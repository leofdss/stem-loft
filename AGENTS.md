# StemLoft — contributor & agent guide

A Tauri v2 desktop app (Rust core + Angular presentation) for musicians
practicing along with a song's stems. Full pitch: [README.md](README.md).
Rust is the single source of truth; Angular only presents state and sends
intent — never decides anything on its own.

This file is for **everyone who changes the repo**: a person, or any AI
coding tool (Claude Code, opencode, Codex, Cursor, Copilot, a local model
behind [`scripts/local_llm.py`](scripts/local_llm.py), ...). It's written
once, in plain Markdown, so no reader needs a particular tool to follow it.

**Before starting any task, survey the project.** Don't assume familiarity
from a previous session — every new contributor (human or agent) starts
cold. At minimum: read this file in full, skim the headers of
[`docs/architecture.md`](docs/architecture.md) for the area you're changing,
find the work in [OpenSpec](openspec/README.md), and check the playbook
table below. Every rule here exists because skipping that survey already
produced a real problem once (a stale doc, a contradicted decision, a rule
only one tool knew about).

## Agent-agnostic rule

**Absolute — no exceptions, no "just this once".** Every piece of project
knowledge and every enforced rule lives in a tool-neutral place:

| Kind | Lives in |
|---|---|
| Instructions & rules | this file, [`docs/`](docs/architecture.md) |
| Behavior specs & planned work | [`openspec/`](openspec/README.md) |
| Task playbooks | [`.agents/skills/<name>/SKILL.md`](#playbooks) — plain Markdown |
| Checks & enforcement | [`scripts/`](#scripts), [`.githooks/`](#checks-what-runs-when) |

A tool-specific location — `CLAUDE.md`, `.claude/`, `.opencode/`,
`.cursor/`, `GEMINI.md`, `.github/copilot-instructions.md`, ... — may only
hold a **thin adapter**: a pointer to the canonical file, or a hook that
translates the tool's JSON into a call to `scripts/`. Concretely:

- A rule that only one tool enforces is not a project rule yet. Put the
  logic in `scripts/`, then wire as many tools to it as you like.
- Never copy a playbook into a tool's folder — link or symlink it
  (`.claude/skills` is a symlink to `.agents/skills`).
- A new tool integration adds adapters only; it never changes what a human
  contributor has to read or run.
- If a tool can't consume the canonical format, write the smallest adapter
  that makes it, never a parallel version of the content.

[`scripts/check_agent_agnostic.py`](scripts/check_agent_agnostic.py) checks
this mechanically (instruction files point to `AGENTS.md` and stay short,
adapters reference an existing canonical file, hook adapters delegate to
`scripts/` and stay small, playbooks don't use one tool's mechanics) and
runs in [`scripts/verify.sh`](scripts/verify.sh).

## Source of truth, in order

1. [`docs/architecture.md`](docs/architecture.md) — every architecture
   decision and its rationale. Wins over anything below if they disagree.
2. [`openspec/specs/`](openspec/README.md) — the behavior the app already
   commits to, one capability per folder.
3. [`openspec/changes/`](openspec/README.md) — planned work: one change per
   task (`task-NNN-<slug>`), each with a proposal, acceptance-criteria
   tasks, and the behavior it adds; [`openspec/board.toml`](openspec/board.toml)
   holds each change's status, branch, dependencies and playbooks. Check it
   before assuming a piece of work hasn't been scoped, or duplicating one.
4. The playbooks below — operational summaries of `architecture.md`. If a
   playbook and `architecture.md` diverge, `architecture.md` wins and the
   playbook gets corrected, not the other way around.

## Playbooks

Read the matching `.agents/skills/<name>/SKILL.md` before working in its
area. Tools that understand the `SKILL.md` format load them automatically;
everyone else just opens the file.

| Playbook | Read it when |
|---|---|
| `add-ipc-contract` | Adding/changing a `Command` (Angular→Rust) or `Event` (Rust→Angular) — the most common entry point for new features |
| `rust-domain-module` | Creating a new Rust module or domain struct, or unsure "is this too idiomatic?" |
| `realtime-audio-safety` | Touching anything near `cpal`, decoding, mixing, or the playback loop |
| `atomic-persistence` | Adding or changing any disk write (project `.json`, waveform cache, future `.cho`) |
| `chordpro-format` | Implementing/testing the `.cho` parser |
| `remote-build-offload` | Before running `cargo`/`npm` checks, builds or tests — routes them to the optional remote host, falls back to local silently |
| `canvas-to-mermaid` | Converting an Obsidian `.canvas` into Markdown+Mermaid docs |
| `review-rust-core` | Reviewing any change under `src-tauri/` — do it before calling the change done |
| `review-angular-shell` | Reviewing any change under `ui/` — do it before calling the change done |
| `openspec-propose`, `openspec-apply-change`, `openspec-update-change`, `openspec-explore`, `openspec-sync-specs`, `openspec-archive-change` | Working with OpenSpec changes — generated by `openspec init --tools agents`, see [openspec/README.md](openspec/README.md) for this repo's extra rules (bilingual artifacts, [`scripts/openspec_archive.py`](scripts/openspec_archive.py) instead of a bare `openspec archive`) |

The two review playbooks are meant to be applied **proactively**, by
whoever finishes a change (a person, or a separate reviewing agent), not
only on request.

## Checks: what runs when

Every check is a script; tools and git hooks only call it.

| When | What | How |
|---|---|---|
| After editing a file | `cargo check` (`.rs`), `tsc --noEmit` and `prettier --check` (`ui/`), npm baseline (`package.json`), doc links + bilingual parity (`.md`), kanban freshness (`openspec/`), "does README need an update?" notes | [`scripts/check_changed.py <paths>`](scripts/check_changed.py) |
| Before running a shell command | No `npm install <pkg>` (no third-party npm packages, see [`architecture.md#stack-and-platform`](docs/architecture.md#stack-and-platform)); Conventional Commits | [`scripts/guard_command.py "<cmd>"`](scripts/guard_command.py) |
| On `git commit` | the same per-file checks on staged files; Conventional Commits | [`.githooks/`](.githooks/pre-commit) — enable once per clone: `git config core.hooksPath .githooks` |
| Before calling a task done / opening a PR | everything: script tests, agent-agnostic rule, links, parity, npm baseline, OpenSpec validation, kanban, cargo check/clippy/test, npm build/test, prettier | [`scripts/verify.sh`](scripts/verify.sh) (`--quick` skips cargo/npm) |

Findings are feedback, not obstacles: fix what they report before calling
the change done. `check_changed.py` separates **errors** (objectively wrong;
exit 1) from **notes** (a judgment call no script can make, like "README.md
may need a matching update").

Adding a new npm dependency, or a lint builder (`ng add angular-eslint`
means a new dependency), needs the maintainer's approval first — don't add
one unilaterally.

## Scripts

Prefer these over re-deriving a check by hand. All are deterministic: same
input, same output, whoever runs them.

- [`verify.sh`](scripts/verify.sh) — the full pass; the definition of green.
- [`check_changed.py`](scripts/check_changed.py) — per-file feedback after an edit.
- [`guard_command.py`](scripts/guard_command.py) / [`check_commit_msg.py`](scripts/check_commit_msg.py) — shell-command policy / Conventional Commits.
- [`check_agent_agnostic.py`](scripts/check_agent_agnostic.py) — the rule above.
- [`check_doc_links.py`](scripts/check_doc_links.py) — every internal Markdown link/anchor, against GitHub's slug algorithm.
- [`check_bilingual_parity.py`](scripts/check_bilingual_parity.py) — heading and invariant-code-block parity of every EN/pt-BR pair, plus missing OpenSpec translations.
- [`check_npm_deps.py`](scripts/check_npm_deps.py) — `ui/package.json` against the approved baseline.
- [`check_remote_image.py`](scripts/check_remote_image.py) — the remote build container runs the same image as the local one (`--fix` recreates it). See `remote-build-offload`.
- [`openspec.sh`](scripts/openspec.sh) — the pinned OpenSpec CLI.
- [`openspec_archive.py`](scripts/openspec_archive.py) — archive a finished change in both languages.
- [`openspec_to_kanban.py`](scripts/openspec_to_kanban.py) — the only writer of [`docs/kanban.html`](docs/kanban.html); never hand-edit that file.
- [`local_llm.py`](scripts/local_llm.py) — send a prompt to the local-model server by role.
- [`llm_task.py`](scripts/llm_task.py) — let a local model implement one file against a check it can't change (frozen tests, stateless iterations, stall detection).
- [`translate_md.py`](scripts/translate_md.py) — draft a pt-BR translation without letting the model touch document structure.

Their own tests live in [`scripts/tests/`](scripts/tests/test_workflow_scripts.py)
(`python3 -m unittest discover -s scripts/tests`).

## Choosing who does the work

Whoever orchestrates (a person or an AI agent) doesn't need to write every
line itself — but only hand work to a cheaper model (a small hosted model,
or a local one on the build host) when the task has a **hard, mechanical
feedback loop** that catches a wrong-but-plausible answer:

- **Safe to hand off**: scaffolding-heavy Rust (`cargo check` gates it) or
  Angular (`tsc` gates it) work, test scaffolding the test run itself
  checks, first drafts of pt-BR translations (`check_bilingual_parity.py`
  checks structure; a person still reads the prose), commit messages and
  PR summaries a person reads anyway, and anything a script already
  verifies (`canvas-to-mermaid` is the reference shape: a deterministic
  script does the real work, the model orchestrates and reports).
- **Keep on a stronger model or a person**: anything marked `human` or
  `both` in `openspec/board.toml`, the `.cho` parser's grammar edge cases,
  anything inside the `cpal` real-time callback, and the two review
  playbooks — their whole job is the judgment a compiler can't provide.
- A compiling-but-wrong answer is only caught if something downstream
  checks the "wrong" part — a green `cargo check` doesn't mean a correct
  crossfade. Don't hand off a task in an area with no such check just
  because *some* check exists in the repo.

**Local models.** The optional remote build host can also serve local
models through Ollama on its GPU
([distrobox/remote/README.md](distrobox/remote/README.md#local-models)).
Roles → models are defined once in
[`distrobox/remote/ollama/models.toml`](distrobox/remote/ollama/models.toml)
(`quick`, `code`, `translate`, `heavy`) — call them by role, never by model
name:

```bash
python3 scripts/local_llm.py --status
git diff --cached | python3 scripts/local_llm.py --role quick \
  --system "Write one Conventional Commits subject line for this diff."
```

**Handing a file to a local model** (`scripts/llm_task.py`): write the
tests first and freeze them (`--freeze-from '#[cfg(test)]'`), give the
model the exact API and ideally a skeleton (`--start-from`), and let the
harness loop it against `cargo test`. Expect small local models to get
close and stall on the last few tests, and to miss bugs the tests don't
cover: treat the output as a draft that a stronger model or a person
finishes and reviews (the review playbooks still apply).

Exit status 3 means no local model is configured or reachable — like the
remote build host being absent, that's normal: do the work another way.
The service is on-demand: start it with `distrobox/remote/power.sh up
ollama`, stop it with `power.sh down` when done.

## Working in parallel

- **One change = one branch = one worktree.** Each worker takes one
  `openspec/changes/task-NNN-*` change, works on the branch `board.toml`
  names, in its own git worktree (or clone), and hands back the branch
  with `scripts/verify.sh` green.
- **Only the orchestrator edits the board** (`openspec/board.toml`,
  `docs/kanban.html`) and archives changes. Workers report status; they
  don't move cards — otherwise every PR conflicts on the same files.
- **Don't parallelize changes that touch the same files.** Check
  `depends_on` and the files a change will touch (e.g. `session.rs`,
  `ipc/commands.rs`) before starting two at once.
- **Power the remote host down when the session ends.** It's usually
  someone's own machine: `distrobox/remote/power.sh down` stops
  the local-model service and the build container. `power.sh up ollama`
  starts the models again when needed; nothing there starts by itself.
- **A worker on the remote build host uses its own clone**, never
  `$STEMLOFT_REMOTE_PATH`: `distrobox/remote/sync.sh` overwrites that
  directory with the local working tree.
- **Review is separate from authorship**: the review playbooks run on every
  branch before merge, and `scripts/verify.sh` is the final gate for
  everyone.

## Testing policy: tests are the source of truth

The acceptance criteria in each change's `tasks.md` are deliberately
explicit about what needs a test — treat that as the floor, not a
suggestion. The test suite (not anyone's memory of how something used to
behave) is what proves a change didn't silently break something else.

- **Every new/changed piece of logic gets a test in the same change.**
  Rust: `#[cfg(test)] mod tests` in the same file — see
  `src-tauri/src/core/persistence.rs`'s `atomic_write` tests for the
  pattern. Angular: a `.spec.ts` beside the file it tests, run through
  Vitest via `@angular/build:unit-test`. Repo scripts: `scripts/tests/`.
- **A task isn't done until its "Unit tests cover ..." criterion actually
  passes** — run `cargo test` / `npm run test -- --watch=false` (directly,
  via `remote-build-offload`, or as part of `scripts/verify.sh`), not just
  `cargo check`/`tsc`. Those only prove the code compiles.
- **Coverage is measured on both sides:** Rust with `cargo llvm-cov
  --summary-only` (or `--html`); Angular with `npm run test` (coverage on
  by default, reports in `ui/coverage/`, gitignored).
- **No hard coverage percentage is enforced yet, on purpose.** Most core
  modules are still stubs. Once a task's real implementation lands, its own
  acceptance criteria are the bar for that code. Don't use "the repo-wide
  percentage is already low" as a reason to skip testing new logic.
- **A failing test is feedback, not an obstacle.** Never delete, skip
  (`#[ignore]`, `.skip()`), or loosen an assertion just to make the suite
  green — find out whether the code or the test is wrong.

## Docs are bilingual, tooling isn't

Every human-facing doc (`README.md`, `TODO.md`, `docs/architecture.md`,
`docs/project-name.md`, `distrobox/README.md`, `distrobox/remote/README.md`,
`ui/README.md`, `openspec/README.md`) and every OpenSpec artifact
(proposal, tasks, design, spec) has a pt-BR counterpart — **edit both in
the same change**; `check_bilingual_parity.py` fails when a structure
drifts or an OpenSpec translation is missing. See
[openspec/README.md](openspec/README.md#languages) for where OpenSpec's
pt-BR files live.

This file, `.agents/skills/*/SKILL.md`, tool adapters, and `scripts/` stay
**English-only, on purpose**: they're operational, read by every kind of
tool, and translating them risks breaking skill matching, which keys off
the English description text.

Commit messages follow [Conventional
Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`,
`refactor:`, `test:`, `chore:`, ...), branches `<type>/<slug>` with the
same type.

## Dev environment

Reproducible Arch Linux dev container: see [README.md — Dev
environment](README.md#dev-environment) and
[`distrobox/README.md`](distrobox/README.md). An optional per-developer
remote build host (with optional local models) may exist
(`distrobox/remote/`) — go through the `remote-build-offload` playbook to
check for one rather than assuming either way.
