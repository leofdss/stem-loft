# Remote build offload

> **Maintenance note:** this doc has a Brazilian Portuguese sibling at
> [`README.pt-BR.md`](./README.pt-BR.md). Whenever one is updated, update
> the other in the same change — don't let the two drift apart.

Optional: if you have a second, faster always-on Linux machine on the same
Tailscale/LAN network, with [Distrobox](https://distrobox.it/) and Podman
installed, you can offload the CPU-heavy parts of the dev loop — `cargo
check`/`clippy`/`test`/`build`/`llvm-cov` for the Rust core, `npm run
build`/`test` for the Angular UI — to it, while the Tauri window itself
(and audio playback, which needs local ALSA/PipeWire) keeps running on your
own machine.

## Day-to-day flow

Nothing changes about how you edit code — you still work on the files
locally, in your own editor. What this adds is a second, separate track for
a different moment of the loop:

| | Runs where | What it's for | How |
|---|---|---|---|
| **Actually running the app** (window, audio) | Local | Seeing/hearing what you built | `cargo tauri dev`, same as always |
| **Just checking it compiles / tests pass** | Remote host | Fast feedback, no local CPU spent | `distrobox/remote/check.sh` |

`cargo tauri dev` has to stay local — it opens a window on your screen and
plays audio through your sound card, neither of which the remote host has
access to. But most of the time you're not testing the window or the audio,
you're asking "does this compile? did I break a test?" — that's pure CPU,
no GUI needed, and that's the part this hands off to the remote host's
cores instead of your own machine's.

A typical loop:

1. Edit code locally, as usual.
2. Before firing up the real app, run `distrobox/remote/check.sh`. It
   pushes your working tree to the remote host and runs
   `scripts/verify.sh` there — the repo's full verification pass — so a wrong type
   or a broken test (Rust or Angular) shows up in well under a minute,
   without your machine compiling `webkit2gtk` locally just to find out.
3. Once that's clean (or whenever you want to actually see the change),
   enter the local container and run `cargo tauri dev` like before.

To skip the "did I remember to sync" step, leave a terminal running
`distrobox/remote/sync.sh --watch` in the background — it keeps pushing
your edits every 2s, so `distrobox/remote/run.sh --dir src-tauri cargo
check` is always checking current code.

## Why this exists

Compiling `src-tauri` pulls in `webkit2gtk`, `tao`, `muda`, `soup3` and the
rest of the GTK/WebKit stack — a cold `cargo check` alone runs tens of
seconds even on decent hardware, and a full `cargo build`/`cargo test` cycle
is worse. If a beefier machine is sitting idle on the network, there's no
reason the local machine's fans should spin for that instead.

## Prior art

This isn't a novel idea — it's the same pattern a few existing tools already
formalize:

- [`cargo-remote`](https://github.com/sgeisler/cargo-remote) — a `cargo`
  subcommand that rsyncs a Rust project to a remote host over SSH, runs
  `cargo` there, and copies `target/` back. This directory is a small,
  Tauri/Angular-aware reimplementation of the same idea (`sync.sh` +
  `run.sh` instead of a single `cargo remote` command), so it can drive both
  halves of the project (`src-tauri` and `ui`) and reuse the project's own
  `distrobox.ini`/`Containerfile` instead of assuming a bare-metal toolchain
  on the remote host.
- [Tauri Discussion #6357, "Developing a Tauri app over
  SSH"](https://github.com/tauri-apps/tauri/discussions/6357) — someone
  with a weak laptop and a powerful desktop hit the exact same GUI-app
  version of this problem: the WebView content (the Angular/vite dev
  server) can be served remotely since it's just HTTP, while the native
  shell (window + audio) still has to run locally.
- [Mozilla's `sccache`](https://github.com/mozilla/sccache) — the
  industrial-strength version of "farm compilation out to other machines,"
  used for Firefox's own build. Worth reaching for instead of this if the
  team ever grows enough machines to want a shared distributed cache rather
  than one dedicated remote host.
- [Mutagen](https://mutagen.io/) — a more robust alternative to `sync.sh`'s
  rsync-polling loop, if continuous low-latency two-way sync (rather than a
  push-only loop) becomes worth the extra dependency.

## Setup

1. On the remote host: Distrobox + Podman installed, `~/Projects/` writable,
   and enough disk headroom for a Rust target dir + node_modules (redirect
   `CARGO_TARGET_DIR` via `~/.cargo/config.toml`'s `[build] target-dir` and
   npm's cache via `~/.npmrc`'s `cache=` if the root disk is tight).
2. `distrobox assemble create --file distrobox/distrobox.ini` on the remote
   host, from a synced copy of this repo — creates the same `stemloft`
   container documented in `distrobox/README.md`, just on that machine.
3. `cp distrobox/remote/host.local.example distrobox/remote/host.local` and
   set `STEMLOFT_REMOTE_HOST` (`user@host`) and `STEMLOFT_REMOTE_PATH` (where
   the project lives on that host, e.g. `~/Projects/stem-loft`) — gitignored,
   this is per-developer, not shared.
4. `python3 scripts/check_remote_image.py --fix` from your local machine —
   makes the remote `stemloft` container run the **exact image your local
   one runs** (pulled by digest, every other `distrobox.ini` setting kept).
   The local container is the reference; `latest` isn't, since it names a
   different image on each machine as soon as one of them pulls a newer
   build. `check.sh` runs the same script (without `--fix`) before every
   pass and stops on a mismatch, so a stale remote image can't quietly
   give you different results than your own machine would. Whenever you
   recreate your local container, rerun this step.

## Usage

```bash
distrobox/remote/sync.sh              # one-shot push of the working tree
distrobox/remote/sync.sh --watch      # keep pushing every 2s while you edit

distrobox/remote/run.sh --dir src-tauri cargo check
distrobox/remote/run.sh --dir src-tauri cargo clippy --all-targets
distrobox/remote/run.sh --dir src-tauri cargo test
distrobox/remote/run.sh --dir src-tauri cargo llvm-cov --summary-only
distrobox/remote/run.sh --dir ui npm run build
distrobox/remote/run.sh --dir ui npm run test -- --watch=false

distrobox/remote/check.sh             # sync + scripts/verify.sh (the full pass), all in one
```

`sync.sh` excludes `.git`, `target/`, `node_modules/`, `dist/`, `.angular/`
— only source goes over the wire; build artifacts are regenerated remotely
and never round-trip back by default (nothing here copies `target/` back,
unlike `cargo-remote` — the point is fast feedback on errors/warnings/test
results, not running the remote-built binary locally, since the ALSA/window
handles it'd need aren't there).

### Parallel worktrees

Every script here works the same from a linked `git worktree` (one per
change, AGENTS.md "Working in parallel"; the gitignored `.worktrees/`
folder is the usual place for them), through `lib.sh`:

- `host.local` is read from the main checkout when the worktree has none.
- The worktree syncs to its own remote directory,
  `<STEMLOFT_REMOTE_PATH>-worktrees/<worktree folder name>`, so parallel
  workers never overwrite each other's files or wait on the same cargo
  lock. Its first sync seeds `src-tauri/target/` and `ui/node_modules/`
  from the main checkout's remote copy, so it skips a cold build.
- Running several `check.sh` at once, set `STEMLOFT_KEEP_BUILD_UP=1` so
  the first one to finish doesn't stop the container under the others,
  and run `power.sh down build` yourself once they're all done.

Old worktree directories on the host aren't cleaned up automatically:
`ssh <host> rm -rf <STEMLOFT_REMOTE_PATH>-worktrees/<name>` once a branch
is merged.

## Local models

> **Experimental — not part of the default workflow.** On a GPU with
> ~8 GB of VRAM, the models that fit didn't finish real tasks even inside
> `scripts/llm_task.py`, so delegation goes to a smaller hosted model
> instead (AGENTS.md, "Choosing who does the work"). Everything below is
> kept working for retesting on bigger hardware.

The same host can serve local language models on its GPU, so any
contributor or AI agent can hand a mechanically-checked chore to a model
that costs nothing per call, has no usage quota, and needs no account with
any AI provider. Requests to it stay inside the tailnet — but that only
keeps the code private if no hosted model is in the same workflow: when a
hosted assistant writes the tests or reviews the result, the code has
already left the machine. It runs
[Ollama](https://ollama.com/) as a rootless Podman Quadlet service
(`stemloft-ollama`), API bound to the host's Tailscale IP only, cloud
models disabled. The GPU is detected by Ollama's ROCm build (AMD); for
another vendor, change the image tag and devices in
[`ollama/ollama.container.in`](ollama/ollama.container.in).

```bash
distrobox/remote/ollama/install.sh        # install/update the service + pull the models
distrobox/remote/power.sh up ollama       # start it for a work session
python3 scripts/local_llm.py --status     # is it up, which models are pulled/loaded
distrobox/remote/power.sh down            # done: stop ollama and the build container
```

**Nothing runs while nobody works.** The build host is usually someone's
own machine, used for other things too, so the service never starts at
boot (`ollama.container.in` has no `[Install]` section), unloads an idle
model after 5 minutes, and `check.sh` stops the build container when its
pass ends. Whoever finishes a work session — a person or an AI agent —
runs `distrobox/remote/power.sh down`; `power.sh status` shows what's
running and how much VRAM is in use.

Then add the URL `install.sh` prints to your `host.local`
(`export STEMLOFT_OLLAMA_URL=http://<tailscale-ip>:11434`). Callers ask for
a **role**, never a model — roles live in
[`ollama/models.toml`](ollama/models.toml), whose defaults fit a GPU with
about 8 GB of VRAM (edit it for yours, re-run `install.sh`):

| Role | Default model | Use it for |
|---|---|---|
| `quick` | `qwen3:8b` | commit messages, PR summaries, classifying a diff |
| `translate` | `qwen3:8b` (same model, no reload) | pt-BR drafts via `scripts/translate_md.py` |
| `code` | `qwen2.5-coder:7b` | mechanical edits gated by `cargo check`/`tsc`/tests, e.g. through `scripts/llm_task.py` |
| `heavy` | `qwen3-coder:30b` | longer sessions over bigger contexts (a MoE that may spill from VRAM into RAM) |

Design rules the scripts enforce, because a model won't reliably follow
them when asked:

- **The model never sees structure it must not change.**
  `scripts/translate_md.py` sends only prose segments (JSON with a schema)
  and rejects a segment whose code spans or `SHALL`/`MUST` changed;
  `scripts/llm_task.py` freezes the tests (`--freeze-from`) and strips line
  numbers a model echoes back.
- **One model never writes both the code and the test that judges it** —
  otherwise it can "fix" correct code to match its own wrong expectation.
- **Hung checks are killed**, not waited on: `llm_task.py` runs the check
  in its own process group under a timeout.
- **Output is a draft.** A script, a stronger model or a person checks it
  before it lands (see "Choosing who does the work" in
  [`AGENTS.md`](../../AGENTS.md)).

Keep measurements of your own host — speeds, which models fit, lab runs —
in `distrobox/remote/lab.local.md` (gitignored, like `host.local`): they
describe one machine, not the project.

## What this deliberately doesn't do

- **Doesn't run the actual `cargo tauri dev` window remotely.** Audio
  playback (cpal/ALSA) and the WebView window both need to be on the
  machine you're sitting at. This offloads compilation, not execution.
- **Doesn't sync back automatically.** If a remote `cargo check` changes
  `Cargo.lock` (a new dependency), copy it back by hand — that's a
  deliberate choice someone should look at, not something to sync blind.
