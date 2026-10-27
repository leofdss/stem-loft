# StemLoft

> **Maintenance note:** this doc has a Brazilian Portuguese sibling at
> [`README.pt-BR.md`](./README.pt-BR.md). Whenever one is updated, update
> the other in the same change — don't let the two drift apart.

A practice tool for musicians, built around looping a section of a song and
mixing its stems while you play along.

> 🚧 **Under active development — not usable yet.** The Tauri v2 + Angular
> scaffold is in place (`src-tauri/`, `ui/`) — it builds and the two layers
> are wired together — but every domain module in the Rust core is still a
> stub (`import_stems`, `play`, `set_markers`, and the rest return "not yet
> implemented"). See [TODO.md](TODO.md) for what's left before the app does
> anything real.

## For musicians (once it's usable)

StemLoft is aimed at **beginner musicians** learning a song from its stems
— the isolated tracks for each instrument (guitar, bass, drums, vocals...).
The core idea: mark a section on the timeline and StemLoft repeats it
continuously, at the recording's original tempo, so you can practice your
part over and over, playing along with or in place of an instrument from
the original recording.

What that will let you do:

- **Loop a hard section on repeat** using a start/end marker — no manual
  rewinding.
- **Mix each stem yourself**: volume, mute, and solo per instrument — for
  example, mute the part you're learning and play over the rest of the
  band.
- **See the chords and tablature synced to playback**, from a plain-text
  score file you can also read and edit outside the app.
- **(Planned)** upload a full track and get it automatically split into
  stems, instead of needing them pre-separated.

None of this is usable yet — see the status callout above.

## For contributors

### What is this

Desktop app built on **Tauri v2**: a **Rust** core (audio, state,
persistence) in [`src-tauri/`](src-tauri), embedded with an **Angular**
presentation layer in [`ui/`](ui) running inside Tauri's WebView. See
[`docs/architecture.md`](docs/architecture.md) for the full design —
components, data flow, and the reasoning behind each decision.

### Why

The project deliberately minimizes supply-chain attack surface: the Angular
layer uses **only native framework features**, no third-party npm packages,
and Rust is the single source of truth for state and logic — Angular only
presents what Rust reports and sends user intent, never deciding or holding
its own logic. Details and rationale in
[`docs/architecture.md#stack-and-platform`](docs/architecture.md#stack-and-platform).

### Tech stack

- **Tauri v2** — desktop shell and IPC bridge (`Commands`/`Events`) between the two layers.
- **Rust** — the core: audio engine (`symphonia`, `rtrb`, `cpal`), state, persistence, and the `.cho` (extended ChordPro) parser.
- **Angular** — presentation only, native Signals for state, no third-party packages.

### Dev environment

The repo ships a reproducible Arch Linux dev container (Distrobox/Podman)
with Rust, Node, Python, Claude Code, and the Tauri build dependencies
already installed. With Distrobox and Podman on the host:

```bash
distrobox assemble create --file distrobox/distrobox.ini
distrobox enter stemloft
```

Details and the reasoning behind the image in
[`distrobox/README.md`](distrobox/README.md). Once inside the container,
from the repo root:

```bash
cargo tauri dev
```

(the Tauri CLI is installed as a `cargo` binary, not an npm package — see
[`docs/architecture.md#stack-and-platform`](docs/architecture.md#stack-and-platform)
for why the Angular side avoids npm dependencies beyond what Angular itself
brings). This starts the Angular dev server and opens the app's window —
today, the empty-state screens the scaffold ships with, since none of the
Rust core's domain logic is implemented yet.

### Debugging in VS Code

Open the repo root in the container's VS Code (`code .`, or the "Visual
Studio Code (on stemloft)" launcher) and install the recommended
extensions it offers (rust-analyzer, CodeLLDB, Angular Language Service).
`.vscode/launch.json` has three configurations:

- **Tauri: debug Rust core** — starts the Angular dev server, builds the
  dev binary, and runs it under CodeLLDB: breakpoints in `src-tauri/` hit
  while you use the real app window.
- **Rust: debug unit tests** — the core's `#[cfg(test)]` tests under
  CodeLLDB.
- **Angular: debug unit tests** — the `.spec.ts` suite (Vitest + jsdom)
  under VS Code's Node debugger: breakpoints in `ui/src/` hit as the tests
  exercise that code.

Breakpoints in Angular code *inside the running app window* can't come
from VS Code on Linux: Tauri renders through WebKitGTK, which doesn't speak
the Chrome DevTools protocol VS Code's JavaScript debugger needs (only
Windows' WebView2 does). Use the WebView's own inspector instead —
right-click in the window → **Inspect Element** (enabled in debug builds)
→ **Sources**: the TypeScript files are there through the dev server's
source maps, and breakpoints set there pause the app. Both debuggers work
at once: run **Tauri: debug Rust core** and open the inspector in its
window to follow a `Command` from Angular into Rust.

### Docs

- [`docs/architecture.md`](docs/architecture.md) — full architecture: components, IPC contract, the `.cho` format, data flows.
- [`docs/project-name.md`](docs/project-name.md) — why the project is called StemLoft.
- [`TODO.md`](TODO.md) → [`openspec/`](openspec/README.md) — the v1 task
  board: one OpenSpec change per task (proposal, acceptance criteria, the
  behavior it adds), status in `openspec/board.toml`, visual board in
  [`docs/kanban.html`](docs/kanban.html).
- [`AGENTS.md`](AGENTS.md) — how to work on the repo, for people and any AI
  coding tool alike: rules, playbooks, checks, who does which work.
- [`distrobox/README.md`](distrobox/README.md) — reproducible Arch Linux dev environment (Distrobox/Podman): what's in the image (`Containerfile`) and how the container is created from it (`distrobox.ini`).

### Contributing

Contributions are welcome. Start with [`TODO.md`](TODO.md) for what's
currently open, and [`docs/architecture.md`](docs/architecture.md) for the
decisions already settled — changes should follow that design, or update
the doc alongside the code if they deliberately change it. The project is
GPL-3.0-or-later (see [License](#license) below), so anything contributed
stays open.

Read [`AGENTS.md`](AGENTS.md) before your first change — it's the single
contributor guide, for people and AI coding tools alike. Commit messages
follow [Conventional Commits](https://www.conventionalcommits.org/)
(`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`, ...). Every rule
is a script anyone can run, never something only one tool knows:

```bash
git config core.hooksPath .githooks   # once per clone: commit-msg + pre-commit checks
scripts/verify.sh                     # the full pass, before every PR
```

The repo is deliberately **AI-tool-agnostic**: playbooks are plain Markdown
in `.agents/skills/`, checks live in `scripts/`, and tool folders such as
`.claude/` only hold thin adapters that point back to them — use whatever
editor or assistant you like (see "Agent-agnostic rule" in `AGENTS.md`).

## License

Copyright (C) 2026 Leonardo Farias de Souza Silva

Distributed under the [GNU General Public License v3.0 or later](./LICENSE)
— the same license family used by Inkscape, GIMP, Blender, and Audacity:
anyone can use, study, modify, and redistribute the code (commercial use
included), but every distributed derivative work must stay under the same
license, with source available — no one can take this project, close it
off, and sell a proprietary version.
