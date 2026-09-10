# Development image (Distrobox)

The `Containerfile` in this folder defines StemLoft's development image:
Arch Linux (`quay.io/toolbx/arch-toolbox`, maintained by the Toolbx
project — Distrobox's sibling, same family of tools) with Python,
Node.js + fnm, Rust, git, Claude Code, and VS Code.

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

The workflow builds and publishes to `ghcr.io/<owner>/stemloft-dev` on every
push that changes the `Containerfile`, weekly (Mondays, 06:00 UTC), and on
demand (`workflow_dispatch`). It only runs once this repository exists on
GitHub and the push actually lands there — today the project is still local
only.

**One-time manual step, on the first publish:** packages on GHCR are born
private by default, even in a public repository. After the first published
build, go to the `stemloft-dev` package's *Package settings* (at
`github.com/<owner>/stemloft-dev/pkgs/container/stemloft-dev`, the
*Package settings* tab) and switch visibility to *Public* — only needed
once.

## Next step

This `Containerfile` only solves the **image**. Configuring Distrobox
itself (the `assemble` file, extra mounts, container name) is the next
step, once the image is published.
