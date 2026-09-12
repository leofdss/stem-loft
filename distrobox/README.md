# Development environment (Distrobox)

This folder holds the two pieces of StemLoft's dev environment: the
`Containerfile` (what's *inside* the image) and `distrobox.ini` (how the
container is *created* from it).

## Quick start

Requires [Distrobox](https://distrobox.it/) and Podman on the host. From
the repository root:

```bash
distrobox assemble create --file distrobox/distrobox.ini
distrobox enter stemloft
```

The first command pulls the published image and creates the `stemloft`
container; it's safe to re-run — if the container already exists it says so
and does nothing. To recreate it from scratch (to pick up a newer image,
for example), add `--replace`; only the container is discarded, since all
the work lives in the host's `HOME`, which is shared.

Inside the container, the repository is at the same path as on the host,
and `git`, `claude`, and `code` are ready to use with the host's own
credentials.

## The image

The `Containerfile` in this folder defines StemLoft's development image:
Arch Linux (`quay.io/toolbx/arch-toolbox`, maintained by the Toolbx
project — Distrobox's sibling, same family of tools) with Python,
Node.js + fnm, Rust, git, Claude Code, VS Code, and fish/starship for an
interactive shell.

Target runtime: **Podman**. Publishing: **GitHub Container Registry (ghcr.io)**,
public, built via GitHub Actions (`.github/workflows/build-dev-image.yml`).

## Why publish the image

1. **External audit** — anyone outside the project can read the
   `Containerfile`, see exactly what goes into the image, and reproduce the
   build (`podman build -f distrobox/Containerfile .`) without depending on
   anything pre-packaged by a third party.
2. **Long-term reliability** — the build runs weekly (Arch is a rolling
   release), so the published image carries recent security patches instead
   of freezing whatever packages existed the day someone remembered to
   rebuild.

## Relevant decisions in the Containerfile

- **Base `quay.io/toolbx/arch-toolbox`, not `archlinux:base-devel`.** It's
  the Arch image maintained by the Toolbx/Distrobox ecosystem itself —
  already comes with `base-devel`, `git`, and `sudo` ready. Even so, the
  `Containerfile` still needs to run
  `pacman-key --init && pacman-key --populate archlinux` before the first
  `pacman -Syu`: this base image doesn't ship with the local signing master
  key generated, and without it the `archlinux-keyring` sync hook fails
  ("no secret key available to sign with") while that package updates.
- **Nothing in `/home`.** Distrobox mounts the host's `HOME` over the
  container's `HOME` — anything installed in `/home/<user>` during the
  build becomes invisible (or worse, silently overwritten) once the
  container actually runs. Every package goes to `/usr` (via `pacman`/`npm`
  as root) or `/opt` (VS Code).
- **Rust via the repo package (`rust`), not `rustup`.** `rustup` by default
  lives in `$HOME/.cargo`/`$HOME/.rustup`, which would fall into the same
  trap as the point above. Arch's `rust` package is updated frequently
  (rolling release), which already covers the project's policy of keeping
  toolchains on the latest stable version (the same logic applied to
  Tauri/Angular, see `docs/architecture.md#stack-and-platform`). If a
  target the system package doesn't cover is ever needed (e.g., Android),
  `rustup` can be installed later, per user, inside the shared `HOME`
  itself — in that case, "lives in HOME" is the expected behavior, not an
  image-build accident.
- **Both `fnm` and the system `nodejs`/`npm`.** The repo's `nodejs`/`npm`
  give an always-ready system version; `fnm` (also from the repo) is
  available to pin a Node version per project whenever that's needed
  someday. `fnm`'s version directory (`~/.local/share/fnm` by default) sits
  in `HOME` on purpose — that's the tool's native behavior, and the data
  belongs to the user, not the image.
- **Claude Code via `npm install -g` as root, during the build.** No
  official package in the Arch repos; installed as root during the build,
  which places it in `/usr/lib/node_modules` + `/usr/bin/claude` — outside
  `HOME`.
- **VS Code via Microsoft's official tarball, not AUR.** Also has no
  official package in the Arch repos. The most common alternative would be
  AUR (`visual-studio-code-bin`), but that ties the image to a third-party
  PKGBUILD's maintenance — against the goal of an auditable "works long
  term" image. Instead, the `Containerfile` downloads the generic tarball
  published by Microsoft itself and extracts it to `/opt/vscode`, with a
  symlink at `/usr/local/bin/code`.
- **Authentication (git, Claude Code) gets no special handling here.**
  `~/.gitconfig`, `~/.git-credentials`, `~/.claude/`, etc. already live in
  `HOME` — since Distrobox shares `HOME` with the host by default, logging
  in once on the host keeps working even after recreating the container.
  There's nothing to configure in the image for this to work.
- **No `USER`/user creation in the image.** Distrobox creates the
  container's user (same UID/GID/name as the host) and grants sudo when the
  container is created, not at image build time — that's why the `sudo`
  package is installed, but no user account is created here.

## Building and testing locally

```bash
podman build -t stemloft-dev -f distrobox/Containerfile .
podman run --rm -it stemloft-dev bash -lc \
  'git --version && python --version && node --version && fnm --version && rustc --version && claude --version && code --version'
```

## Publishing (GitHub Actions)

The workflow builds and publishes to `ghcr.io/leofdss/stemloft-dev` on every
push that changes the `Containerfile`, weekly (Mondays, 06:00 UTC), and on
demand (`workflow_dispatch`). It has already run: the image is published and
publicly pullable, tagged `latest` plus a dated `YYYYMMDD-<short sha>` tag
per build (useful to pin a specific build in `distrobox.ini` when needed).

Packages on GHCR are born private by default, even in a public repository —
that visibility switch was a one-time manual step on the first publish (the
`stemloft-dev` package's *Package settings* tab, at
`github.com/leofdss/stem-loft/pkgs/container/stemloft-dev`) and is already
done. It only comes back if the package is ever deleted and republished.

## The container (`distrobox.ini`)

`distrobox.ini` is a [`distrobox assemble`](https://distrobox.it/usage/distrobox-assemble/)
manifest: it declares the container in the repo instead of leaving it to a
long `distrobox create` line typed by hand, so the environment is
reviewable in a diff and identical for everyone who runs the command — the
same reasoning behind publishing the image.

Relevant decisions in it:

- **Container name: `stemloft`** (the section header), so the command to
  get in is `distrobox enter stemloft`.
- **`pull=true`.** Always checks the registry for a newer `latest` when
  creating the container — otherwise recreating it would silently reuse a
  months-old local copy and defeat the weekly rebuild.
- **`init=false`** (no systemd). Nothing in here is a service, and
  `--init` makes Distrobox *skip* mounting the host's `XDG_RUNTIME_DIR` —
  which is exactly where the Wayland, PipeWire, and PulseAudio sockets the
  Tauri WebView and the audio engine need live.
- **`entry=false`.** No `.desktop` entry in the host's application menu:
  this is a development container entered from a terminal, not an app. A
  GUI app that *should* appear in the host menu can be exported per user,
  from inside the container, with `distrobox-export --app code`.
- **`nvidia=false`.** StemLoft is an audio app — the WebView renders
  through the regular GTK/WebKit stack and nothing in the core needs the
  GPU. Worth revisiting only if automatic stem separation ever runs
  locally instead of through an external API.
- **No extra volumes, deliberately.** Distrobox already mounts, with no
  configuration: the host's `HOME` (so the repository, `~/.gitconfig`, and
  `~/.claude` are the same files inside and out — the sharing the
  `Containerfile` relies on), `/dev` and `/sys` (so `/dev/snd` reaches
  ALSA/cpal) along with the host's supplementary groups (`audio`,
  `video`, …), the `XDG_RUNTIME_DIR` sockets, and the whole host
  filesystem under `/run/host` for anything outside `HOME` (stems on
  another mount point, say). A `volume=` line is only needed for a path
  that must appear at the *same* path inside the container as on the host.
- **No `additional_packages`.** Every tool belongs in the
  `Containerfile`, where it's part of the published, auditable image.
  Installing from the manifest instead would make each developer's
  container quietly different from the image everyone else pulls.

To check what the manifest expands to without creating anything:

```bash
distrobox assemble create --dry-run --file distrobox/distrobox.ini
```

## Running a container off a locally built image

`distrobox.ini` points at the published `ghcr.io/leofdss/stemloft-dev:latest`.
To run the container off a local build instead — to test a `Containerfile`
change before pushing it — build it as in
[Building and testing locally](#building-and-testing-locally), then edit the
manifest to use that image and skip the registry:

```ini
image=localhost/stemloft-dev:latest
pull=false
```

Create it with `--replace` to swap an existing `stemloft` container for one
built on the local image, and revert those two lines before committing.
