# TODO — Implementation task board

> **Maintenance note:** this doc has a Brazilian Portuguese sibling at
> [`TODO.pt-BR.md`](./TODO.pt-BR.md). Whenever one is updated, update
> the other in the same change — don't let the two drift apart.

All the pre-implementation decisions this doc used to track (concurrency
model, audio crates, atomic writes, Angular state management, error UX,
Tauri version, `.cho` editing scope) are **resolved** and documented as
design notes in [`docs/architecture.md`](docs/architecture.md) — see the
resolution log in git history if you need the original discussion. This doc
is now the **v1 implementation board**: concrete tasks, sized for a single
human or AI-agent session, each with acceptance criteria and the branch that
implements it.

## How to use this board

- Columns are `## Backlog` → `## In Progress` → `## Review` → `## Done`.
  Move a task by cutting its `### TASK-NNN` block to the target column.
- Every task has a stable `TASK-NNN` id (never reused, even if a task is
  dropped), an **Area**, who it's realistically **suitable for**, the
  **skill(s)** from the root [`CLAUDE.md`](CLAUDE.md) table to load first,
  the **branch** that implements it, and what it **depends on**.
- Branch names follow `<type>/<slug>`, mirroring the [Conventional
  Commits](https://www.conventionalcommits.org/) type of the commit(s) that
  land on it (mostly `feat/`; `test/` and `chore/` show up for non-feature
  tasks) — enforced for commit messages by `bash_guard.py`, not for branch
  names, but keep them aligned.
- "Suitable for: AI agent" means the task is self-contained enough to hand
  to one with this repo's skills/hooks loaded. "Human" marks tasks that need
  a judgment call a hook can't check (does this sound right on real
  headphones, does this feel right in the hand) — an AI agent can still take
  a first pass, but a human has to sign off before `Done`.
- **Any AI agent that changes this file's (or `TODO.pt-BR.md`'s) task or
  column structure must regenerate [`docs/kanban.html`](docs/kanban.html)
  before considering the change done:**

  ```bash
  python3 scripts/todo_to_kanban.py --html
  ```

  This is enforced by the `kanban_sync_reminder.py` hook, but don't rely on
  spotting the reminder — just run it as part of the edit. The command reads
  both `TODO.md` and `TODO.pt-BR.md` and writes the whole page
  deterministically; it's a pure function of those two files, so the output
  never depends on who runs it or how they read this doc — never hand-edit
  `docs/kanban.html` itself. You can also get just the raw Mermaid block
  (e.g. to paste elsewhere) with:

  ```bash
  python3 scripts/todo_to_kanban.py TODO.md            # prints to stdout
  python3 scripts/todo_to_kanban.py TODO.md -o docs/kanban.md
  ```

  See [`scripts/todo_to_kanban.py`](scripts/todo_to_kanban.py) for how it
  parses this file — it's driven entirely by the `##`/`###` structure below,
  so keep new tasks in that shape.

## Backlog

### TASK-001 — Session/State Manager & AppState wiring
- **Area:** Rust core — Session/State Manager
- **Suitable for:** AI agent
- **Skill(s):** `rust-domain-module`, `add-ipc-contract`
- **Branch:** `feat/session-app-state`
- **Depends on:** —

Replace the `session.rs` stub with a real `AppState` behind a single
`Arc<Mutex<AppState>>` (or at most one `Mutex` per module), and wire the
manual-DI constructors for the domain modules Session routes `Commands` to.

**Acceptance criteria:**
- [ ] `AppState` matches [the concurrency-model design note](docs/architecture.md#design-note-concurrency-model-for-shared-state) — no channels, no actor
- [ ] `Session` holds only "which project is open"; every `Command` handler delegates to the module owning that domain, per [Session as a thin router](docs/architecture.md#design-note-session-as-a-thin-router)
- [ ] Unit tests cover `Session` dispatching a `Command` to the correct module (a fake module records the call), without re-testing that module's own logic
- [ ] `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [ ] Reviewed by `rust-core-reviewer`

### TASK-002 — Project Persistence: load/save, atomic writes, schemaVersion
- **Area:** Rust core — Project Persistence
- **Suitable for:** AI agent
- **Skill(s):** `atomic-persistence`
- **Branch:** `feat/project-persistence`
- **Depends on:** TASK-001

Implement `persistence.rs`: read/write the project `.json` per the
[schema](docs/architecture.md#project-versioning), write-to-temp+rename with
no exceptions, and the three `schemaVersion` cases (same version loads,
lower version runs migrations, higher version errors explicitly).

**Acceptance criteria:**
- [ ] Every write goes through a temp file + `rename`, never a direct write to the final path
- [ ] Writes happen at a debounce/checkpoint, not once per `Command` (see [design note](docs/architecture.md#design-note-when-project-persistence-writes))
- [ ] All three `schemaVersion` cases are covered by a test
- [ ] A simulated crash mid-write leaves the previous file intact (test)
- [ ] `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [ ] Reviewed by `rust-core-reviewer`

### TASK-003 — Stem Importer: manual import, validation, mono upmix
- **Area:** Rust core — Stem Importer
- **Suitable for:** AI agent
- **Skill(s):** `rust-domain-module`
- **Branch:** `feat/stem-importer`
- **Depends on:** TASK-001, TASK-002

Implement manual stem import: sample-rate consistency check, mono→stereo
upmix, and `durationSec` extraction from the file header (no full decode).

**Acceptance criteria:**
- [ ] A stem whose sample rate differs from already-imported stems is rejected with an explicit error, not silently resampled
- [ ] A mono stem is upmixed to stereo at import time — `Audio Engine` never sees a mono buffer
- [ ] `durationSec` is read from the header and stored per [Consistency across a project's stems](docs/architecture.md#consistency-across-a-projects-stems)
- [ ] Unit tests cover: a sample-rate mismatch is rejected, a mono stem is upmixed to stereo, and `durationSec` is read correctly, each against a real small fixture file
- [ ] `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [ ] Reviewed by `rust-core-reviewer`

### TASK-004 — Loop & Marker Manager
- **Area:** Rust core — Loop & Marker Manager
- **Suitable for:** AI agent
- **Skill(s):** `rust-domain-module`
- **Branch:** `feat/loop-marker-manager`
- **Depends on:** TASK-003

`set_markers`/state for the looped section, validated against the project's
duration.

**Acceptance criteria:**
- [ ] `endSec` beyond the project's duration (longest stem's `durationSec`) is rejected, not silently clamped
- [ ] Current markers are exposed to `Audio Engine` for loop playback
- [ ] Unit tests cover: a marker within duration is accepted, `endSec` beyond duration is rejected
- [ ] `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [ ] Reviewed by `rust-core-reviewer`

### TASK-005 — Audio Engine: cpal output + symphonia decode + rtrb pipeline
- **Area:** Rust core — Audio Engine
- **Suitable for:** Both — AI agent drafts, human verifies on a real output device
- **Skill(s):** `realtime-audio-safety`
- **Branch:** `feat/audio-engine-playback`
- **Depends on:** TASK-001, TASK-003

Wire `symphonia` decoding → `rtrb` ring buffer → `cpal` callback for basic
playback (no loop/crossfade yet — that's TASK-006).

**Acceptance criteria:**
- [ ] Decoding and any I/O happen outside the `cpal` callback; the callback only reads already-prepared buffers
- [ ] No allocation, lock, or I/O inside the callback (checked against the `realtime-audio-safety` checklist)
- [ ] Unit tests cover the decode → ring-buffer handoff in isolation, without needing a real `cpal` device
- [ ] A human confirms audible, glitch-free playback on at least one real device
- [ ] `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [ ] Reviewed by `rust-core-reviewer`

### TASK-006 — Loop crossfade at the boundary
- **Area:** Rust core — Audio Engine
- **Suitable for:** Both — AI agent drafts, human judges audio quality
- **Skill(s):** `realtime-audio-safety`
- **Branch:** `feat/audio-engine-crossfade`
- **Depends on:** TASK-004, TASK-005

Short crossfade (a few ms) at the loop boundary instead of a hard cut, per
[the design note](docs/architecture.md#design-note-crossfade-at-the-loop-boundary).

**Acceptance criteria:**
- [ ] The start-of-loop buffer is prepared before the callback reaches `endSec`
- [ ] The crossfade is pure math over already-decoded samples inside the callback — no new allocation/I/O
- [ ] Unit tests cover the crossfade math (fade curve) over synthetic sample buffers, independent of `cpal`
- [ ] A human listens for clicks/artifacts at the loop boundary on at least one real project and confirms none
- [ ] `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [ ] Reviewed by `rust-core-reviewer`

### TASK-007 — Waveform peak computation + disk cache
- **Area:** Rust core — Audio Engine / Project Persistence
- **Suitable for:** AI agent
- **Skill(s):** `atomic-persistence`, `realtime-audio-safety`
- **Branch:** `feat/waveform-cache`
- **Depends on:** TASK-002, TASK-003

**Acceptance criteria:**
- [ ] Peaks are computed once (on import, or on first open if no cache exists) and written through the atomic-write path
- [ ] The cache is invalidated when its stem is re-imported/replaced
- [ ] `waveform_ready` serves peaks from the disk cache when present, without redecoding
- [ ] Unit tests cover: peaks computed on first access, the disk cache reused (not recomputed) on the next open, and invalidated after re-import
- [ ] `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [ ] Reviewed by `rust-core-reviewer`

### TASK-008 — Mixer state (volume/mute/solo), real-time-safe propagation
- **Area:** Rust core — Audio Engine
- **Suitable for:** AI agent
- **Skill(s):** `realtime-audio-safety`
- **Branch:** `feat/mixer-state`
- **Depends on:** TASK-005

**Acceptance criteria:**
- [ ] A volume/mute/solo change writes to an atomic/double-buffer the callback reads — never a `Mutex` it can block on
- [ ] A mixer change marks the project dirty for the debounced checkpoint (TASK-002), not a write per tick
- [ ] Unit tests cover the volume/mute/solo state transitions and the dirty flag being set on a change
- [ ] `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [ ] Reviewed by `rust-core-reviewer`

### TASK-009 — IPC Commands surface
- **Area:** Communication Bridge — Tauri IPC
- **Suitable for:** AI agent
- **Skill(s):** `add-ipc-contract`
- **Branch:** `feat/ipc-commands`
- **Depends on:** TASK-001

**Acceptance criteria:**
- [ ] The `Commands` queue lives entirely in the Rust core; nothing in Angular queues, dedups, or reorders
- [ ] Each `Command` handler only dispatches to the module owning that domain — no coordination logic in the handler itself
- [ ] Unit tests cover each `Command` handler dispatching to its owning module, independent of the Tauri runtime
- [ ] `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [ ] Reviewed by `rust-core-reviewer` and `angular-shell-reviewer` (call sites)

### TASK-010 — IPC Events surface (playback_progress, waveform_ready, transport_state_changed, audio_error)
- **Area:** Communication Bridge — Tauri IPC
- **Suitable for:** AI agent
- **Skill(s):** `add-ipc-contract`
- **Branch:** `feat/ipc-events`
- **Depends on:** TASK-005, TASK-007

**Acceptance criteria:**
- [ ] Each event matches the [events table](docs/architecture.md#events-table) exactly (producer, payload, consumers)
- [ ] `audio_error` is actually reachable in manual testing (e.g. disconnect the output device)
- [ ] Unit tests cover each event's payload shape against the [events table](docs/architecture.md#events-table)
- [ ] `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [ ] Reviewed by `rust-core-reviewer`

### TASK-011 — `.cho` parser (Score Metadata Manager)
- **Area:** Rust core — Score Metadata Manager
- **Suitable for:** AI agent
- **Skill(s):** `chordpro-format`
- **Branch:** `feat/cho-parser`
- **Depends on:** —

**Acceptance criteria:**
- [ ] Every grammar rule in the `chordpro-format` skill (one chord per line, `{t:}` anchors, tablature grammar, `startSec`/`endSec` resolution) has a passing unit test
- [ ] An unknown directive is ignored, not fatal
- [ ] `score_parse_error` carries the offending line and message without halting the rest of the app
- [ ] `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [ ] Reviewed by `rust-core-reviewer`

### TASK-012 — `score_loaded` event + real-time chord/beat slice
- **Area:** Rust core / IPC — Score Metadata Manager
- **Suitable for:** AI agent
- **Skill(s):** `add-ipc-contract`, `chordpro-format`
- **Branch:** `feat/score-events`
- **Depends on:** TASK-011

**Acceptance criteria:**
- [ ] `score_loaded` fires once per parse with `tempo`/`time`/`tuning` from the header
- [ ] The per-tick real-time payload carries only the active chord/beat, not the whole score (see [Real-time visual state](docs/architecture.md#real-time-visual-state-angular))
- [ ] Unit tests cover the chord/beat slice selection for a few `positionSec` values against a small fixed score
- [ ] `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [ ] Reviewed by `rust-core-reviewer`

### TASK-013 — Stem Import Screen (Angular)
- **Area:** Angular — Stem Import Screen
- **Suitable for:** Both
- **Skill(s):** `add-ipc-contract`
- **Branch:** `feat/ui-stem-import-screen`
- **Depends on:** TASK-003, TASK-009

**Acceptance criteria:**
- [ ] Fires the import `Command` immediately, with no client-side queue or optimistic update
- [ ] The triggering control stays disabled until that `Command`'s response arrives
- [ ] Vitest covers: the `Command` fires on user action, and the control disables until the response arrives
- [ ] `npm run build` (tsc) and `prettier --check` clean (via `remote-build-offload`)
- [ ] Reviewed by `angular-shell-reviewer`

### TASK-014 — Transport Controls (Angular)
- **Area:** Angular — Transport Controls
- **Suitable for:** Both
- **Skill(s):** `add-ipc-contract`
- **Branch:** `feat/ui-transport-controls`
- **Depends on:** TASK-009, TASK-010

**Acceptance criteria:**
- [ ] Reflects `transport_state_changed`; never assumes a state before the event arrives
- [ ] Vitest covers the displayed state updating for each `transport_state_changed` variant
- [ ] `npm run build` (tsc) and `prettier --check` clean (via `remote-build-offload`)
- [ ] Reviewed by `angular-shell-reviewer`

### TASK-015 — Timeline + Waveform (Angular)
- **Area:** Angular — Timeline + Waveform
- **Suitable for:** Both
- **Skill(s):** `add-ipc-contract`
- **Branch:** `feat/ui-timeline-waveform`
- **Depends on:** TASK-007, TASK-009, TASK-010

**Acceptance criteria:**
- [ ] Renders `waveform_ready` peaks and the `playback_progress` position
- [ ] Marker/loop selection sends `set_markers` and waits for the response before moving the marker on screen
- [ ] Vitest covers rendering from injected `waveform_ready`/`playback_progress` fixtures, and the marker not moving before the `set_markers` response
- [ ] `npm run build` (tsc) and `prettier --check` clean (via `remote-build-offload`)
- [ ] Reviewed by `angular-shell-reviewer`

### TASK-016 — Stem Mixer (Angular)
- **Area:** Angular — Stem Mixer
- **Suitable for:** Both
- **Skill(s):** `add-ipc-contract`
- **Branch:** `feat/ui-stem-mixer`
- **Depends on:** TASK-008, TASK-009

**Acceptance criteria:**
- [ ] Dragging a fader doesn't trigger a persistence write per tick (verified against TASK-002/TASK-008's debounce)
- [ ] Vitest covers the fader firing the mixer `Command` with no extra client-side logic (queue, debounce, or dedup) of its own
- [ ] `npm run build` (tsc) and `prettier --check` clean (via `remote-build-offload`)
- [ ] Reviewed by `angular-shell-reviewer`

### TASK-017 — Chord/Tab View (Angular, read-only)
- **Area:** Angular — Chord/Tab View
- **Suitable for:** Both
- **Skill(s):** `add-ipc-contract`, `chordpro-format`
- **Branch:** `feat/ui-chord-tab-view`
- **Depends on:** TASK-012, TASK-010

**Acceptance criteria:**
- [ ] Renders chords/tablature/lyrics synced to the playback position
- [ ] `score_parse_error` puts only this view into an error state — the rest of the screen keeps working
- [ ] Vitest covers chord/beat highlighting from a `ChordBeatStream`-shaped fixture, and the isolated error state on `score_parse_error`
- [ ] `npm run build` (tsc) and `prettier --check` clean (via `remote-build-offload`)
- [ ] Reviewed by `angular-shell-reviewer`

### TASK-018 — Error Modal (Angular)
- **Area:** Angular — Error Modal
- **Suitable for:** Both
- **Skill(s):** `add-ipc-contract`
- **Branch:** `feat/ui-error-modal`
- **Depends on:** TASK-010, TASK-012

**Acceptance criteria:**
- [ ] `audio_error` and `score_parse_error` open the modal with the event's cause and message
- [ ] Dismissing the modal doesn't pause or undo anything already running in the core
- [ ] Vitest covers the modal opening with the right cause/message for both error events, and dismiss not touching any other state
- [ ] `npm run build` (tsc) and `prettier --check` clean (via `remote-build-offload`)
- [ ] Reviewed by `angular-shell-reviewer`

### TASK-019 — End-to-end manual QA pass (v1 golden path)
- **Area:** Whole app
- **Suitable for:** Human
- **Skill(s):** `run`
- **Branch:** `test/v1-golden-path`
- **Depends on:** TASK-001 – TASK-018

**Acceptance criteria:**
- [ ] Import stems → set loop markers → play with crossfade → adjust mixer → close and reopen the project restores state → `.cho` shows chords synced to playback, all on a real device
- [ ] Any bug found is filed as a new `Backlog` task here, not patched ad hoc inside this task

## In Progress

## Review

## Done

## Icebox (post-v1, not yet broken into tasks)

Out of scope for the board above, but already acknowledged in
[Current status and future work](docs/architecture.md#current-status-and-future-work) —
don't start these without first turning them into their own `TASK-NNN`
entries with acceptance criteria:

- Automatic stem separation (`Separation Client` in the core, `Automatic
  Separation` in the UI, the external `Stem Separation API`) — needs an
  async job pattern (polling or webhook) not yet designed.
- Editing chords/tablature in `Chord/Tab View`, with the core writing the
  `.cho` back (reusing the atomic-write pattern from TASK-002).
