# TODO — Pending decisions before implementation

Decisions that [`architecture.md`](docs/architecture.md) deliberately left open: these
aren't architecture bugs, they're design/implementation choices that still need to be
pinned down before coding starts. Check them off with `[x]` as they're resolved (and
reflect the decision in `architecture.md` if it affects documented behavior).

## Rust core

- [x] **Concurrency model for shared state** — resolved: the model that's **simplest
  to reason about**, not the most sophisticated — `Arc<Mutex<AppState>>` (or at most
  one `Mutex` per module), no channels, no actor. A `Command` handler locks, changes
  what it needs, releases the lock; the `cpal` real-time thread never touches this
  `Mutex` (it only reads the already-prepared atomics/ring buffers — a rule that
  already existed). Rationale: the target is one person clicking buttons, not a
  concurrent server — lock contention at this volume isn't a real problem. Only
  consider something more elaborate if profiling shows actual contention, not as
  speculative optimization. See
  [design note](docs/architecture.md#design-note-concurrency-model-for-shared-state).
- [x] **Audio crates** — resolved, prioritizing running on Linux/Windows/macOS/Android
  with the least possible friction: `symphonia` for decoding (pure Rust, no native
  dependency — decodes identically on all four platforms), `rtrb` for the buffer
  between decoding and the real-time callback (SPSC, wait-free — narrower and better
  aligned with the use case than `ringbuf`), `cpal` for audio output (already covers
  all four platforms, including Android via Oboe). Implementation point of attention
  (doesn't change the choice): `cpal`'s Oboe backend on Android needs the context/JVM
  that Tauri mobile exposes — wiring to do at initialization, not domain logic. See
  [design note](docs/architecture.md#design-note-audio-crates-cross-platform).
- [x] **Behavior at the loop boundary** — resolved: a **short crossfade** (a few ms),
  not a hard cut. Consequence for the `Audio Engine`: the start of the loop (from
  `startSec`) needs to be ready in its own buffer *before* the real-time callback
  reaches the end of the loop, since the crossfade blends the section ending with the
  one starting — there's no decoding that on the fly inside the callback. Exact
  crossfade duration and curve (linear/equal-power) remain an implementation detail.
  See
  [design note](docs/architecture.md#design-note-crossfade-at-the-loop-boundary).
- [x] **Stem channels (mono/stereo)** — resolved: channels are always **stereo**
  within the project. A mono stem is upmixed (channel duplicated to L/R) by `Stem
  Importer` at import time, not on every playback — unlike sample rate, there's no
  rejection here, because duplicating mono into stereo has no quality ambiguity the
  way resampling would. `Audio Engine` never deals with mono buffers. See
  [Consistency across a project's stems](docs/architecture.md#consistency-across-a-projects-stems).

## Persistence

- [x] **Atomic writes** — resolved: **every** disk write from `Project Persistence`
  is atomic (write-to-temp-file + rename), no exceptions — not just the project's
  `.json` at the debounced checkpoint, but also the waveform cache and, in the
  future, the `.cho`. See
  [design note](docs/architecture.md#design-note-when-project-persistence-writes).
- [x] **Waveform cache** — resolved: **cached to disk**, alongside the project (the
  `stems[].waveformCache` field in the `.json`, written atomically like everything
  else) — not recalculated every time the project opens. Reason: performance on weak
  devices is a project requirement, and decoding the whole stem to recompute peaks on
  every open is exactly the kind of cost this avoids. The cache is invalidated if the
  stem is re-imported/replaced. See
  [design note](docs/architecture.md#design-note-waveform-cache-on-disk).

## IPC / Angular

- [x] **State management in Angular** — resolved: **Signals**, native to Angular —
  no NgRx, no other external (npm) package beyond what the framework itself already
  provides, to minimize supply-chain attack surface (see [Stack and platform](docs/architecture.md#stack-and-platform)).
  How `Commands` connect to components was already decided, with one correction: the
  `Commands` queue lives in the **Rust core**, not in Angular — Angular only presents
  the state Rust reports and sends user intent, with no queue/logic of its own (the
  same "all intelligence lives in Rust" reasoning used to minimize the cost of a
  future framework swap). The effect for the user stays the same: the control that
  fired a `Command` stays disabled until the response (preventing a double click),
  other controls remain free to fire their own `Commands`, which the core processes
  in sequence. See
  [design note](docs/architecture.md#design-note-commands-queue-in-the-rust-core).
- [x] **Error UX** — resolved: a **modal**, not a toast or persistent banner, for
  `audio_error` and `score_parse_error`. The modal is only about presentation — it
  doesn't pause or undo anything already running in the core; `score_parse_error`
  still doesn't halt the rest of the app (dismiss the modal, `Chord/Tab View` sits in
  an error state, the rest of the screen keeps working normally).
  See [design note](docs/architecture.md#design-note-error-presentation-modal).

## Scope / setup

- [x] **Tauri version** — resolved: **Tauri v2**, and the project's policy is to
  keep Tauri and Angular on the latest stable version at all times (not pinned once
  and left alone) — reinforcing the same attack-surface-minimization goal as the
  other Angular decisions (no third-party packages, native Signals). See [Stack and platform](docs/architecture.md#stack-and-platform).
  Consequence that still holds: the Rust core must not depend on API details of a
  specific Tauri version — the `Commands`/`Events` bridge is treated as replaceable,
  not as a fixed part of the core's design (the same separation that would, in the
  limit, allow swapping the presentation layer itself, e.g. for Flutter).
- [x] **Editing the `.cho` in v1** — resolved: the app will both read **and** update
  the `.cho` as the user edits chords in the frontend, but that editing isn't part of
  v1 — v1 is read-only, as already documented. The difference is that "writing" is no
  longer dismissed as forever out of the architecture's scope: it's a real feature
  planned for after v1 (see
  [Current status and future work](docs/architecture.md#current-status-and-future-work)). When it
  lands, the `.cho` write flow will likely reuse the atomic-write pattern
  (write-to-temp-file + rename) already decided in **Atomic writes** above, since both
  cases boil down to "don't corrupt a project text file if the app crashes mid-write."
