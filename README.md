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

### Docs

- [`docs/architecture.md`](docs/architecture.md) — full architecture: components, IPC contract, the `.cho` format, data flows.
- [`docs/project-name.md`](docs/project-name.md) — why the project is called StemLoft.
- [`TODO.md`](TODO.md) — pending decisions and what's left before implementation.
- [`distrobox/README.md`](distrobox/README.md) — reproducible Arch Linux dev environment (Distrobox/Podman): what's in the image (`Containerfile`) and how the container is created from it (`distrobox.ini`).

### Contributing

Contributions are welcome. Start with [`TODO.md`](TODO.md) for what's
currently open, and [`docs/architecture.md`](docs/architecture.md) for the
decisions already settled — changes should follow that design, or update
the doc alongside the code if they deliberately change it. The project is
GPL-3.0-or-later (see [License](#license) below), so anything contributed
stays open.

This repo also ships Claude Code agents, skills, and hooks
(`.claude/`) that encode these rules so an AI assistant working on the code
follows them automatically — for example, a `PostToolUse` hook nudges the
agent to keep this README in sync whenever `docs/architecture.md`,
`TODO.md`, or the dev-environment docs change.

## License

Copyright (C) 2026 Leonardo Farias de Souza Silva

Distributed under the [GNU General Public License v3.0 or later](./LICENSE)
— the same license family used by Inkscape, GIMP, Blender, and Audacity:
anyone can use, study, modify, and redistribute the code (commercial use
included), but every distributed derivative work must stay under the same
license, with source available — no one can take this project, close it
off, and sell a proprietary version.
