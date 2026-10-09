#!/usr/bin/env python3
"""Check that the remote build host's `stemloft` container runs the same
image as the local one -- and, with --fix, recreate it so it does.

The remote build offload (distrobox/remote/) only gives trustworthy
feedback if the remote container is the same environment as the local one:
a stale remote image once meant a tool the local container had (and the
Containerfile promised) was simply missing remotely. "Same image" is
decided here by comparing image IDs reported by `podman`, never by reading
dates or tags -- `latest` names a different image on each machine as soon
as one of them pulls a newer build.

The local container is the reference. --fix never changes the local
container; it recreates the remote one from the exact digest the local
container's image was pulled at, reusing every other setting in
distrobox/distrobox.ini (only the `image=` line is swapped for that
digest). The remote HOME -- the synced project, ~/.cargo, caches -- is
untouched; only the container is replaced.

Usage:
    scripts/check_remote_image.py          # compare only
    scripts/check_remote_image.py --fix    # recreate the remote container on mismatch

Exit codes:
    0  same image on both sides (or --fix made them match)
    1  different images (and --fix wasn't passed, or didn't fix it)
    2  couldn't compare: no distrobox/remote/host.local, host unreachable,
       or no `stemloft` container on one side
"""

from __future__ import annotations

import argparse
import re
import shlex
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Optional

REPO_ROOT = Path(__file__).resolve().parent.parent
HOST_LOCAL = REPO_ROOT / "distrobox" / "remote" / "host.local"
DISTROBOX_INI = REPO_ROOT / "distrobox" / "distrobox.ini"
CONTAINER = "stemloft"
SSH_OPTS = ["-o", "BatchMode=yes", "-o", "ConnectTimeout=5"]


def read_remote_host() -> Optional[str]:
    """STEMLOFT_REMOTE_HOST from host.local, sourced by bash exactly the
    way run.sh/sync.sh source it."""
    if not HOST_LOCAL.is_file():
        return None
    result = subprocess.run(
        ["bash", "-c", 'source "$1" && printf %s "$STEMLOFT_REMOTE_HOST"', "_", str(HOST_LOCAL)],
        capture_output=True,
        text=True,
    )
    return result.stdout.strip() or None


def local_podman() -> Optional[list[str]]:
    """How to reach the local host's podman: directly, or -- when this runs
    inside the distrobox container itself -- through distrobox-host-exec."""
    if shutil.which("podman"):
        return ["podman"]
    if shutil.which("distrobox-host-exec"):
        return ["distrobox-host-exec", "podman"]
    return None


def run_local(args: list[str]) -> Optional[str]:
    podman = local_podman()
    if podman is None:
        return None
    result = subprocess.run(podman + args, capture_output=True, text=True)
    return result.stdout.strip() if result.returncode == 0 else None


def run_remote(host: str, command: str, stdin: Optional[str] = None) -> subprocess.CompletedProcess:
    # Wrapped in `sh -c` so it runs the same whatever the remote login
    # shell is (fish, zsh, ...).
    return subprocess.run(
        ["ssh", *SSH_OPTS, host, "sh -c " + shlex.quote(command)],
        input=stdin,
        capture_output=True,
        text=True,
    )


def remote_output(host: str, command: str) -> Optional[str]:
    result = run_remote(host, command)
    return result.stdout.strip() if result.returncode == 0 else None


def container_image_id_cmd() -> list[str]:
    return ["container", "inspect", CONTAINER, "--format", "{{.Image}}"]


def local_image_id() -> Optional[str]:
    return run_local(container_image_id_cmd())


def remote_image_id(host: str) -> Optional[str]:
    return remote_output(host, "podman " + shlex.join(container_image_id_cmd()))


def local_image_ref(image_id: str) -> Optional[str]:
    """The `repo@sha256:...` reference the local image was pulled at -- the
    one pull reference that resolves to this exact image anywhere."""
    out = run_local(["image", "inspect", image_id, "--format", "{{range .RepoDigests}}{{println .}}{{end}}"])
    if not out:
        return None
    return out.splitlines()[0].strip() or None


def pin_image(ini_text: str, image_ref: str) -> str:
    """distrobox.ini with its `image=` line replaced by `image_ref`;
    everything else (init, nvidia, entry, pull, ...) kept as-is."""
    pinned, count = re.subn(r"(?m)^image=.*$", f"image={image_ref}", ini_text)
    if count != 1:
        raise ValueError(f"expected exactly one image= line in {DISTROBOX_INI}, found {count}")
    return pinned


def recreate_remote(host: str, image_ref: str) -> bool:
    ini = pin_image(DISTROBOX_INI.read_text(), image_ref)
    script = (
        'tmp="$(mktemp --suffix=.ini)" && cat > "$tmp" && '
        'distrobox assemble create --replace --file "$tmp"; '
        'status=$?; rm -f "$tmp"; exit $status'
    )
    result = run_remote(host, script, stdin=ini)
    sys.stdout.write(result.stdout)
    sys.stderr.write(result.stderr)
    return result.returncode == 0


def short(image_id: str) -> str:
    return image_id[:12]


def main() -> int:
    # Keep our lines in order with the remote distrobox output we relay.
    sys.stdout.reconfigure(line_buffering=True)
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--fix", action="store_true", help="recreate the remote container from the local image on mismatch")
    args = parser.parse_args()

    host = read_remote_host()
    if host is None:
        print(f"SKIP -- no remote host configured ({HOST_LOCAL.relative_to(REPO_ROOT)} missing or empty).")
        return 2

    local_id = local_image_id()
    if local_id is None:
        print(f"SKIP -- couldn't read the local `{CONTAINER}` container's image (no podman reachable, or no such container).")
        return 2

    remote_id = remote_image_id(host)
    if remote_id is None:
        if remote_output(host, "true") is None:
            print(f"SKIP -- remote host {host} unreachable.")
            return 2
        print(f"MISSING -- no `{CONTAINER}` container on {host}.")
    elif remote_id == local_id:
        print(f"OK -- local and {host} both run image {short(local_id)}.")
        return 0
    else:
        print(f"MISMATCH -- local runs image {short(local_id)}, {host} runs {short(remote_id)}.")

    if not args.fix:
        print("Run `python3 scripts/check_remote_image.py --fix` to recreate the remote container from the local image.")
        return 1 if remote_id is not None else 2

    image_ref = local_image_ref(local_id)
    if image_ref is None:
        print(f"FAIL -- local image {short(local_id)} has no registry digest to pull it by on {host} (a locally built image?).")
        return 1

    print(f"Recreating `{CONTAINER}` on {host} from {image_ref} ...")
    if not recreate_remote(host, image_ref):
        print("FAIL -- `distrobox assemble create --replace` failed on the remote host (output above).")
        return 1

    remote_id = remote_image_id(host)
    if remote_id == local_id:
        print(f"OK -- local and {host} both run image {short(local_id)}.")
        return 0
    print(f"FAIL -- after recreating, {host} runs {short(remote_id or 'nothing')}, local runs {short(local_id)}.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
