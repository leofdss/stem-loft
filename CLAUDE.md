# StemLoft — agent orientation

A Tauri v2 desktop app (Rust core + Angular presentation) for musicians
practicing along with a song's stems. Full pitch: [README.md](README.md).
Rust is the single source of truth; Angular only presents state and sends
intent — never decides anything on its own.

**Before starting any task: survey the project first.** Don't assume
familiarity from a prior session — every new agent starts cold. At minimum:
read this file in full, skim the headers of
[`docs/architecture.md`](docs/architecture.md) for the section relevant to
what you're changing, and check the skills table below for one that already
covers the task. Every rule below exists because skipping this step already
produced a real problem once (a stale doc, a contradicted decision, a rule
enforced by a hook nobody had documented) — don't reintroduce it.

## Source of truth, in order

1. [`docs/architecture.md`](docs/architecture.md) — every architecture
   decision and its rationale. Wins over anything below if they disagree.
2. [`TODO.md`](TODO.md) — the v1 implementation task board (what's scoped,
   its acceptance criteria, and its status); check it before assuming a
   piece of work hasn't been scoped yet or before duplicating a task.
3. The skills in the table below — operational summaries of
   `architecture.md`, written so you don't have to reconstruct the reasoning
   every time. If a skill and `architecture.md` diverge, `architecture.md`
   wins and the skill should be corrected, not the other way around.

## Skills — load one of these before working in its area

| Skill | Load it when |
|---|---|
| `add-ipc-contract` | Adding/changing a `Command` (Angular→Rust) or `Event` (Rust→Angular) — the most common entry point for new features |
| `rust-domain-module` | Creating a new Rust module or domain struct, or unsure "is this too idiomatic?" |
| `realtime-audio-safety` | Touching anything near `cpal`, decoding, mixing, or the playback loop |
| `atomic-persistence` | Adding or changing any disk write (project `.json`, waveform cache, future `.cho`) |
| `chordpro-format` | Implementing/testing the `.cho` parser |
| `remote-build-offload` | Before running `cargo check/clippy/test/build` or `npm run build/test` — routes heavy work to a remote host if one is configured, falls back to local silently if not |
| `canvas-to-mermaid` | Converting an Obsidian `.canvas` into Markdown+Mermaid docs |

Two review agents exist and should be run proactively (not just on request)
after the matching change: `rust-core-reviewer` after anything under
`src-tauri/`, `angular-shell-reviewer` after anything under `ui/`.

## Hooks already enforcing rules (don't fight them, don't route around them)

- `cargo_check_on_rust_edit.py` — runs `cargo check` after every `.rs` edit
  and hands you the compiler error back. If it doesn't fire, run `cargo
  check` yourself before reporting the task done.
- `bash_guard.py` — blocks `npm install/add <pkg>` (no third-party npm
  packages, see `architecture.md#stack-and-platform`) and blocks `git
  commit` whose message isn't [Conventional
  Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`,
  `refactor:`, `test:`, `chore:`, ...).
- `readme_sync_reminder.py` — nudges you to check `README.md` is still
  accurate after editing `TODO.md`, `docs/architecture.md`,
  `docs/project-name.md`, `distrobox/README.md`, or anything under
  `.claude/agents/`/`.claude/skills/`. This one stays a plain reminder —
  whether the summary is still *accurate* is a judgment call no script can
  make.
- `pt_br_sync_reminder.py` — after editing one half of a `foo.md`/
  `foo.pt-BR.md` pair, runs `scripts/check_bilingual_parity.py` on that
  pair and only speaks up with what it actually found (heading structure
  drift, or a `bash`/`json`/`toml`/`yaml` code block that differs between
  languages — those must never be translated). Silent if the check comes
  back clean; falls back to a generic "go check it" only if the checker
  itself can't run.
- `kanban_sync_reminder.py` — nudges you to run `python3
  scripts/todo_to_kanban.py --html` whenever you edit `TODO.md` or
  `TODO.pt-BR.md`. That script is the only thing that should ever write
  `docs/kanban.html` — it's a pure function of the two TODO files, so
  running it is always correct regardless of who runs it; never hand-edit
  `docs/kanban.html` directly.
- `doc_links_check.py` — after any `.md` edit, runs
  `scripts/check_doc_links.py` across the repo and reports any internal
  link or `#anchor` that doesn't actually resolve. Silent when nothing's
  broken.
- `tsc_check_on_ts_edit.py` — the Angular equivalent of
  `cargo_check_on_rust_edit.py`: runs `tsc --noEmit` after every `.ts`/
  `.html` edit under `ui/` and hands the type error back. No-ops if
  `ui/node_modules` hasn't been installed yet.
- `npm_deps_check.py` — after any `package.json` edit, runs
  `scripts/check_npm_deps.py` and flags a dependency outside the approved
  baseline. Covers the gap `bash_guard.py` can't: a dependency added by
  hand-editing `package.json` and then running a bare `npm install`/`npm
  ci` (no positional package, which `bash_guard.py` allows).
- `prettier_check_on_edit.py` — runs `prettier --check` after every
  `.ts`/`.html`/`.css`/`.scss` edit under `ui/` and hands back which file
  isn't formatted per `ui/.prettierrc`. Report-only, never rewrites the
  file itself. `ng lint` isn't wired up the same way: this project has no
  lint builder configured, and adding one (`ng add angular-eslint`) means a
  new npm dependency — ask the user before doing that, don't add it
  unilaterally.

None of the hooks above block anything — they're all `PostToolUse`
feedback, same convention as `cargo_check_on_rust_edit.py`: the write
already happened, the hook just hands back what it found so you can fix it
before calling the task done. Only `bash_guard.py` (`PreToolUse`) actually
blocks a tool call before it runs.

## Scripts (`scripts/`) — prefer these over re-deriving the check by hand

- `todo_to_kanban.py` — `TODO.md`/`TODO.pt-BR.md` → Mermaid `kanban` block
  or the full `docs/kanban.html` viewer (`--html`). See
  [`TODO.md`](TODO.md#how-to-use-this-board).
- `check_doc_links.py` — validates every internal Markdown link/anchor in
  the repo against GitHub's heading-slug algorithm. Run it yourself after
  a large doc restructure instead of trusting a visual scan.
- `check_bilingual_parity.py` — validates heading-structure and
  invariant-code-block parity between each `foo.md`/`foo.pt-BR.md` pair.
- `check_npm_deps.py` — diffs `ui/package.json`'s dependencies against the
  hardcoded approved baseline (see the script's `ALLOWED` set).

All four are deterministic: same input, same output, regardless of who
runs them or how they read the surrounding prose — that's the point. If
you find yourself re-deriving one of their checks by manually reading a
diff, run the script instead.

## Model selection: when a cheaper model can safely do the work

The orchestrating agent doesn't need to write every line itself — but only
delegate to a cheaper/faster model (e.g. `haiku`) when the task has a
**hard, mechanical feedback loop** it can iterate against without needing
strong judgment to get it right the first time:

- **Safe to delegate** (a hook already catches a wrong-but-plausible
  answer): scaffolding-heavy Rust (`cargo_check_on_rust_edit.py` gates it)
  or Angular (`tsc_check_on_ts_edit.py` gates it) work, and anything
  `todo_to_kanban.py`/the doc checkers already verify mechanically. See
  `canvas-to-mermaid` (`model: haiku` in its agent file) for the reference
  shape: a narrow task, a deterministic script doing the real work, the
  model mostly orchestrating and reporting.
- **Keep on a stronger model**: anything marked `Human` or `Both` in
  `TODO.md`'s "Suitable for" field, the `.cho` parser's grammar edge cases,
  and anything inside the `cpal` real-time callback — a hook can confirm
  the code *compiles*, not that a subtle audio/timing bug isn't hiding in
  code that compiles fine. `rust-core-reviewer` and `angular-shell-reviewer`
  stay on `sonnet`: their entire job is the judgment layer a compiler can't
  provide (is this *architecturally* right, not just syntactically valid).
- A compiling-but-wrong answer from a cheap model is only caught if
  something downstream actually checks the "wrong" part — a green
  `cargo check` doesn't mean a correct crossfade. Don't delegate a task to
  a cheaper model in an area with no such check just because *some* hook
  exists in the repo.

## Testing policy: tests are the source of truth

`TODO.md`'s acceptance criteria are deliberately explicit about what needs
a test — treat that as the floor, not a suggestion. As the codebase grows
past this initial scaffold, the test suite (not a person's memory of how
something used to behave) is what proves a change didn't silently break
something else.

- **Every new/changed piece of logic gets a test in the same change.**
  Rust: `#[cfg(test)] mod tests` in the same file — see
  `src-tauri/src/core/persistence.rs`'s `atomic_write` tests for the
  pattern already in this repo. Angular: a `.spec.ts` beside the file it
  tests, run through Vitest via `@angular/build:unit-test`.
- **A task isn't `Done` until its "Unit tests cover ..." bullet actually
  passes** — run `cargo test` / `npm run test -- --watch=false` (directly
  or via `remote-build-offload`), not just `cargo check`/`tsc`. Those only
  prove the code compiles, not that it does the right thing — see [Model
  selection](#model-selection-when-a-cheaper-model-can-safely-do-the-work)
  above on why that distinction matters for what's safe to delegate.
- **Coverage is measured on both sides:**
  - Rust: `cargo llvm-cov --summary-only` (or `--html` for a browsable
    report) — a `cargo` subcommand installed in the dev container
    (`cargo install cargo-llvm-cov` if it's ever missing locally).
  - Angular: `npm run test` reports coverage by default now
    (`ui/angular.json`'s `test` target has `coverage: true` and
    `@vitest/coverage-v8` installed) — a text summary on every run, plus
    `html`/`lcov` reports written to `ui/coverage/` (gitignored).
- **No hard coverage percentage is enforced yet, on purpose.** Most core
  modules are still stubs — a threshold today would either be trivially
  met or fail on a placeholder that isn't real logic yet. Once a task's
  real implementation lands, its own "Unit tests cover ..." acceptance
  criteria are the bar for that code, not a global number. Don't use "the
  repo-wide percentage is already low" as a reason to skip testing new
  logic, and don't let a stub's low number block an unrelated task.
- **A failing test is feedback, not an obstacle.** Never delete, skip
  (`#[ignore]`, `.skip()`), or loosen an assertion just to make the suite
  green — figure out whether the code or the test is wrong, the same way
  you'd treat a `cargo check`/`tsc` error. This is the same hard-feedback-
  loop principle the Model selection section above is built on; a test
  suite people can't trust stops being a source of truth at all.

## Docs are bilingual, tooling isn't

Every human-facing doc (`README.md`, `TODO.md`, `docs/architecture.md`,
`docs/project-name.md`, `distrobox/README.md`, `distrobox/remote/README.md`,
`ui/README.md`) has a `*.pt-BR.md` sibling — **edit both in the same
change**, the sync hook above will remind you if you forget. This file,
`.claude/agents/*.md`, and `.claude/skills/*/SKILL.md` stay **English-only,
on purpose** — they're read primarily by AI agents, and translating them
risks breaking skill-matching, which keys off the English description text.

## Dev environment

Reproducible Arch Linux dev container: see [README.md — Dev
environment](README.md#dev-environment) and
[`distrobox/README.md`](distrobox/README.md). An optional per-developer
remote build host may exist (`distrobox/remote/`) — always go through the
`remote-build-offload` skill to check for one rather than assuming either
way.
