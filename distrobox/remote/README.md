# Remote build offload

Optional: if you have a second, faster always-on Linux machine on the same
Tailscale/LAN network, with [Distrobox](https://distrobox.it/) and Podman
installed, you can offload the CPU-heavy parts of the dev loop — `cargo
check`/`clippy`/`test`/`build` for the Rust core, `npm run build` for the
Angular UI — to it, while the Tauri window itself (and audio playback, which
needs local ALSA/PipeWire) keeps running on your own machine.

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
   set `STEMLOFT_REMOTE_HOST` to `user@host` (gitignored — this is
   per-developer, not shared).

## Usage

```bash
distrobox/remote/sync.sh              # one-shot push of the working tree
distrobox/remote/sync.sh --watch      # keep pushing every 2s while you edit

distrobox/remote/run.sh --dir src-tauri cargo check
distrobox/remote/run.sh --dir src-tauri cargo clippy --all-targets
distrobox/remote/run.sh --dir ui npm run build

distrobox/remote/check.sh             # sync + check + clippy + test + build, all in one
```

`sync.sh` excludes `.git`, `target/`, `node_modules/`, `dist/`, `.angular/`
— only source goes over the wire; build artifacts are regenerated remotely
and never round-trip back by default (nothing here copies `target/` back,
unlike `cargo-remote` — the point is fast feedback on errors/warnings/test
results, not running the remote-built binary locally, since the ALSA/window
handles it'd need aren't there).

## What this deliberately doesn't do

- **Doesn't run the actual `cargo tauri dev` window remotely.** Audio
  playback (cpal/ALSA) and the WebView window both need to be on the
  machine you're sitting at. This offloads compilation, not execution.
- **Doesn't sync back automatically.** If a remote `cargo check` changes
  `Cargo.lock` (a new dependency), copy it back by hand — that's a
  deliberate choice someone should look at, not something to sync blind.
