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
2. [`TODO.md`](TODO.md) — decisions `architecture.md` deliberately left
   open; check before assuming something is undecided.
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

None of the four doc-sync hooks above block anything — they're
`PostToolUse` feedback, same convention as `cargo_check_on_rust_edit.py`:
the write already happened, the hook just hands back what it found so you
can fix it before calling the task done.

## Scripts (`scripts/`) — prefer these over re-deriving the check by hand

- `todo_to_kanban.py` — `TODO.md`/`TODO.pt-BR.md` → Mermaid `kanban` block
  or the full `docs/kanban.html` viewer (`--html`). See
  [`TODO.md`](TODO.md#how-to-use-this-board).
- `check_doc_links.py` — validates every internal Markdown link/anchor in
  the repo against GitHub's heading-slug algorithm. Run it yourself after
  a large doc restructure instead of trusting a visual scan.
- `check_bilingual_parity.py` — validates heading-structure and
  invariant-code-block parity between each `foo.md`/`foo.pt-BR.md` pair.

All three are deterministic: same input, same output, regardless of who
runs them or how they read the surrounding prose — that's the point. If
you find yourself re-deriving one of their checks by manually reading a
diff, run the script instead.

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
