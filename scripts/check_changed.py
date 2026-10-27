#!/usr/bin/env python3
"""Per-file feedback for whatever just changed -- the tool-neutral core of
every "after an edit" check in this repo.

Usage:
    python3 scripts/check_changed.py <path> [<path> ...]
    python3 scripts/check_changed.py --staged          # files staged in git
    python3 scripts/check_changed.py --json <path>...  # machine-readable

Anyone can run it: a human before committing, the `.githooks/pre-commit`
hook, scripts/verify.sh, or an AI coding tool's post-edit hook (`.claude/hooks/on_edit.py`
is one such thin adapter). No tool owns the logic -- see AGENTS.md
"Agent-agnostic rule".

Two kinds of findings:
  - error -- something is objectively wrong (doesn't compile, not
    formatted, broken link, bilingual drift, dependency outside the
    baseline). Exit status 1.
  - note  -- a judgment call no script can make ("README.md may need a
    matching update"). Printed, but never fails the run on its own.

Each check silently skips when its tool isn't available (no cargo, no
ui/node_modules), same as before: a missing toolchain isn't a finding.
"""
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))
import check_bilingual_parity as parity  # noqa: E402

PRETTIER_EXTENSIONS = (".ts", ".html", ".css", ".scss")

# Paths whose edits may make README.md's summary stale.
README_WATCHED_PREFIXES = (
    "AGENTS.md",
    "TODO.md",
    "docs/architecture.md",
    "docs/project-name.md",
    "openspec/README.md",
    "scripts/verify.sh",
    "distrobox/README.md",
    "distrobox/Containerfile",
    "distrobox/remote/",
    ".agents/skills/",
    "openspec/config.yaml",
)
# Inputs of the generated docs/kanban.html.
KANBAN_INPUT_PREFIXES = ("openspec/changes/", "openspec/board.yaml")


class Findings:
    def __init__(self) -> None:
        self.errors: list[str] = []
        self.notes: list[str] = []

    def error(self, msg: str) -> None:
        self.errors.append(msg.rstrip())

    def note(self, msg: str) -> None:
        self.notes.append(msg.rstrip())


def run(cmd: list[str], cwd: Path, timeout: int = 300) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout)


def find_upwards(start: Path, marker: str) -> Path | None:
    for directory in [start, *start.parents]:
        if (directory / marker).is_file():
            return directory
        if directory == ROOT.parent:
            break
    return None


def rel(path: Path) -> str:
    try:
        return path.relative_to(ROOT).as_posix()
    except ValueError:
        return path.as_posix()


def remote_runner(checkout: Path) -> list[str] | None:
    """`distrobox/remote/run.sh` of `checkout` (the main clone or a linked
    worktree) when its remote build host is configured and answers, after
    syncing the checkout there -- the `remote-build-offload` playbook, so a
    post-edit `cargo check` doesn't spend the local CPU. None means "run
    locally": no host configured, host unreachable, sync failed, or
    STEMLOFT_LOCAL_CHECKS is set (run.sh sets it on the remote side)."""
    if os.environ.get("STEMLOFT_LOCAL_CHECKS"):
        return None
    remote = checkout / "distrobox" / "remote"
    if not (remote / "lib.sh").is_file():
        return None
    host = subprocess.run(
        ["bash", "-c", 'source "$1" && printf %s "${STEMLOFT_REMOTE_HOST:-}"', "_", str(remote / "lib.sh")],
        capture_output=True,
        text=True,
    ).stdout.strip()
    if not host:
        return None
    try:
        probe = subprocess.run(
            ["ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=3", host, "true"],
            capture_output=True,
            timeout=10,
        )
        if probe.returncode != 0 or run([str(remote / "sync.sh")], checkout).returncode != 0:
            return None
    except (OSError, subprocess.TimeoutExpired):
        return None
    return [str(remote / "run.sh")]


def checkout_of(path: Path) -> Path:
    r = subprocess.run(["git", "-C", str(path), "rev-parse", "--show-toplevel"], capture_output=True, text=True)
    return Path(r.stdout.strip()) if r.returncode == 0 and r.stdout.strip() else ROOT


def check_rust(paths: list[Path], f: Findings) -> None:
    crates = {c for p in paths if p.suffix == ".rs" for c in [find_upwards(p.parent, "Cargo.toml")] if c}
    if not crates:
        return
    for crate in sorted(crates):
        cargo = ["cargo", "check", "--quiet", "--message-format=short"]
        checkout = checkout_of(crate)
        runner = remote_runner(checkout)
        if runner is not None:
            r = run([*runner, "--dir", crate.relative_to(checkout).as_posix(), *cargo], checkout)
        elif shutil.which("cargo") is not None:
            r = run(cargo, crate)
        else:
            continue
        if r.returncode != 0:
            f.error(f"cargo check failed in {rel(crate)}:\n{r.stdout}{r.stderr}")


def check_typescript(paths: list[Path], f: Findings) -> None:
    projects = {
        c for p in paths if p.suffix in (".ts", ".html")
        for c in [find_upwards(p.parent, "tsconfig.json")] if c
    }
    for project in sorted(projects):
        tsc = project / "node_modules" / ".bin" / "tsc"
        if not tsc.is_file():
            continue
        cmd = [str(tsc), "--noEmit"]
        if (project / "tsconfig.app.json").is_file():
            cmd += ["-p", "tsconfig.app.json"]
        r = run(cmd, project)
        if r.returncode != 0:
            f.error(f"tsc --noEmit failed in {rel(project)}:\n{r.stdout}{r.stderr}")


def check_prettier(paths: list[Path], f: Findings) -> None:
    for p in paths:
        if p.suffix not in PRETTIER_EXTENSIONS:
            continue
        project = find_upwards(p.parent, ".prettierrc")
        if project is None:
            continue
        prettier = project / "node_modules" / ".bin" / "prettier"
        if not prettier.is_file():
            continue
        r = run([str(prettier), "--check", str(p)], project)
        if r.returncode != 0:
            f.error(
                f"{rel(p)} isn't formatted per {rel(project)}/.prettierrc -- run "
                f"`npx prettier --write {p.relative_to(project)}` from {rel(project)}/."
            )


def check_npm_deps(paths: list[Path], f: Findings) -> None:
    for p in paths:
        if p.name != "package.json":
            continue
        r = run([sys.executable, str(ROOT / "scripts" / "check_npm_deps.py"), str(p)], ROOT)
        if r.returncode != 0:
            f.error(r.stdout + r.stderr)


def check_doc_links(paths: list[Path], f: Findings) -> None:
    if not any(p.suffix == ".md" for p in paths):
        return
    r = run([sys.executable, str(ROOT / "scripts" / "check_doc_links.py")], ROOT)
    if r.returncode != 0:
        f.error(r.stdout + r.stderr)


def check_bilingual(paths: list[Path], f: Findings) -> None:
    seen: set[Path] = set()
    for p in paths:
        other = parity.counterpart_path(p)
        if other is None or not other.is_file():
            continue
        en = other if parity.is_pt_br(p) else p
        if en in seen:
            continue
        seen.add(en)
        r = run([sys.executable, str(ROOT / "scripts" / "check_bilingual_parity.py"), str(en)], ROOT)
        if r.returncode != 0:
            f.error(r.stderr or r.stdout)
        elif p.is_file():
            f.note(f"{rel(other)} is the EN/pt-BR sibling of {rel(p)} -- make sure its prose still says the same thing.")


def check_generated(paths: list[Path], f: Findings) -> None:
    rels = [rel(p) for p in paths]
    if any(r.startswith(KANBAN_INPUT_PREFIXES) for r in rels):
        r = run([sys.executable, str(ROOT / "scripts" / "openspec_to_kanban.py"), "--check"], ROOT)
        if r.returncode != 0:
            f.error(r.stdout + r.stderr)


def check_readme_notes(paths: list[Path], f: Findings) -> None:
    for r in (rel(p) for p in paths):
        if r.endswith(".local.md") or ".local/" in r:  # personal, gitignored notes
            continue
        if r.startswith(README_WATCHED_PREFIXES):
            f.note(f"README.md may need a matching update after this change to {r} -- check its summary is still accurate.")


CHECKS = (
    check_rust,
    check_typescript,
    check_prettier,
    check_npm_deps,
    check_doc_links,
    check_bilingual,
    check_generated,
    check_readme_notes,
)


def staged_paths() -> list[str]:
    r = run(["git", "diff", "--cached", "--name-only", "--diff-filter=ACMR"], ROOT)
    return [line for line in r.stdout.splitlines() if line]


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("paths", nargs="*")
    parser.add_argument("--staged", action="store_true", help="check the files staged in git")
    parser.add_argument("--json", action="store_true", help="print {errors, notes} as JSON")
    args = parser.parse_args(argv)

    raw = args.paths + (staged_paths() if args.staged else [])
    paths = []
    for p in raw:
        path = Path(p)
        path = path if path.is_absolute() else (Path(os.getcwd()) / path)
        paths.append(path.resolve())

    findings = Findings()
    for check in CHECKS:
        try:
            check(paths, findings)
        except (OSError, subprocess.TimeoutExpired) as exc:
            findings.note(f"{check.__name__} couldn't run: {exc}")

    if args.json:
        print(json.dumps({"errors": findings.errors, "notes": findings.notes}))
    else:
        for e in findings.errors:
            print(f"ERROR: {e}")
        for n in findings.notes:
            print(f"note: {n}")
    return 1 if findings.errors else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
