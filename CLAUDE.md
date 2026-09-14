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
  `.claude/agents/`/`.claude/skills/`.
- `pt_br_sync_reminder.py` — nudges you to update the counterpart file
  whenever you edit one half of a `foo.md`/`foo.pt-BR.md` pair (see below).

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
