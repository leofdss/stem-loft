---
name: remote-build-offload
description: How to check whether the optional remote Distrobox/Podman build host (distrobox/remote/) is configured and reachable on this machine, and how to route cargo/npm checks, builds, and tests to it instead of running them locally — with a mandatory, silent local fallback when it isn't. Use before running cargo check/clippy/test/build or npm run build/test, especially a full test-suite pass, so the heavy work runs on the faster host when one exists.
---

# Using the remote build host (if one exists)

This project **optionally** supports offloading `cargo`/`npm`
check-build-test work to a second, faster machine over SSH, using the
scripts in `distrobox/remote/` (see `distrobox/remote/README.md` for the
full design). This host is **per-developer and often absent** — most
checkouts of this repo will not have one configured. Follow this skill
exactly; do not skip the availability check and do not treat an unavailable
remote host as an error.

## Step 1 — Check availability (always do this first, never assume)

```bash
cd <repo root>   # distrobox/remote/host.local is resolved relative to here
if [ -f distrobox/remote/host.local ]; then
  . distrobox/remote/host.local   # sets STEMLOFT_REMOTE_HOST and STEMLOFT_REMOTE_PATH
  if ssh -o BatchMode=yes -o ConnectTimeout=5 "$STEMLOFT_REMOTE_HOST" true 2>/dev/null; then
    REMOTE_AVAILABLE=1
  fi
fi
```

Read the result plainly:

- `distrobox/remote/host.local` **doesn't exist** → no remote host is
  configured on this machine. This is the normal case. Stop here, run
  everything locally, and don't mention it as a problem — it's optional
  infrastructure, not a missing dependency.
- The file exists but the `ssh ... true` check **fails or hangs past 5s**
  → the host is configured but offline/unreachable right now (asleep,
  disconnected, VPN down). Fall back to local immediately. Do not retry in
  a loop, do not increase the timeout, do not block the task waiting for it
  to come back.
- The `ssh` check **succeeds** → the remote host is up; proceed to Step 2.

Never create, edit, or guess a value for `distrobox/remote/host.local`
yourself — it's gitignored and machine-specific. If it's missing, that
means offload is simply not set up here, not that something is broken.

## Step 2 — What to offload vs. what must stay local

| Safe to send remote | Must stay local |
|---|---|
| `cargo check` | `cargo tauri dev` (opens a window) |
| `cargo clippy --all-targets` | anything playing or recording audio |
| `cargo test` | anything the user needs to see or hear |
| `cargo llvm-cov` (coverage report) | |
| `cargo build` | |
| `npm run build` (Angular) | |
| `npm run test -- --watch=false` (Angular, coverage on by default) | |

The rule: if it's pure compilation/verification with no GUI and no audio
device, it's safe to offload. If it needs a screen or a sound card, it has
to run on the machine the user is sitting at — the remote host has neither
attached to this session.

## Step 3 — Run it

With `REMOTE_AVAILABLE=1` from Step 1:

```bash
# One command at a time:
distrobox/remote/run.sh --dir src-tauri cargo check
distrobox/remote/run.sh --dir src-tauri cargo clippy --all-targets
distrobox/remote/run.sh --dir src-tauri cargo test
distrobox/remote/run.sh --dir src-tauri cargo llvm-cov --summary-only
distrobox/remote/run.sh --dir ui npm run build
distrobox/remote/run.sh --dir ui npm run test -- --watch=false

# Or the whole verification pass in one call (syncs first, then runs all four):
distrobox/remote/check.sh
```

These scripts sync the working tree over first (`run.sh` does not sync by
itself — `check.sh` does; if calling `run.sh` directly and the working tree
changed since the last sync, run `distrobox/remote/sync.sh` first). Exit
codes propagate normally: a non-zero exit means the remote `cargo`/`npm`
command failed, exactly as if it had run locally — read and act on the
output the same way you would for a local failure, don't treat a remote
failure as a connectivity problem.

Without `REMOTE_AVAILABLE=1`, just run the equivalent command locally
(`cd src-tauri && cargo test`, `cd ui && npm run build`, etc.) — same
commands, same working directories, just no `distrobox/remote/run.sh`
wrapper and no `--dir` flag.

## Checklist

- [ ] Checked for `distrobox/remote/host.local` before assuming a remote
      host exists?
- [ ] Used a short (~5s) `ssh -o BatchMode=yes` probe rather than assuming
      reachability, and didn't retry/block if it failed?
- [ ] Only sent GUI-free, audio-free work (check/clippy/test/build) to the
      remote host — never `cargo tauri dev` or anything needing the
      screen/speakers?
- [ ] Fell back to the exact same commands run locally, without treating
      "no remote host" as an error to report?
