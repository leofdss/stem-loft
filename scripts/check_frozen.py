#!/usr/bin/env python3
"""Prove that the frozen part of a file (e.g. its contract tests) is
byte-identical to the version committed in a git ref.

Usage:
    python3 scripts/check_frozen.py <file> [--marker '#[cfg(test)]'] [--ref HEAD]

The orchestrator commits the contract (tests + skeleton) before handing a
file to an implementer (any model, any tool -- see the
`implement-against-frozen-tests` playbook). Afterwards this script compares
everything from the first occurrence of --marker to the end of the file in
the working tree against the same region in --ref. Exit 0 = untouched,
1 = changed (prints a unified diff), 2 = can't compare (marker missing,
file not in the ref).

This is what makes "the implementer never edits its own judge" a checked
rule instead of a request: an agent with edit access can be asked not to
touch the tests, but only a diff proves it didn't.
"""
from __future__ import annotations

import argparse
import difflib
import subprocess
import sys
from pathlib import Path

DEFAULT_MARKER = "#[cfg(test)]"


def frozen_region(text: str, marker: str) -> str | None:
    index = text.find(marker)
    return None if index < 0 else text[index:]


def committed_text(path: Path, ref: str) -> str | None:
    top = subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True,
                         cwd=path.parent)
    if top.returncode != 0:
        return None
    rel = path.resolve().relative_to(Path(top.stdout.strip()).resolve()).as_posix()
    shown = subprocess.run(["git", "show", f"{ref}:{rel}"], capture_output=True, text=True,
                           cwd=top.stdout.strip())
    return shown.stdout if shown.returncode == 0 else None


def compare(path: Path, marker: str, ref: str) -> tuple[int, str]:
    before_text = committed_text(path, ref)
    if before_text is None:
        return 2, f"{path} is not in {ref} -- commit the contract before delegating"
    before = frozen_region(before_text, marker)
    after = frozen_region(path.read_text(encoding="utf-8"), marker)
    if before is None or after is None:
        side = "the committed version" if before is None else "the working tree"
        return 2, f"marker {marker!r} not found in {side} of {path}"
    if before == after:
        return 0, f"OK -- frozen part of {path} (from {marker!r}) matches {ref}"
    diff = "".join(difflib.unified_diff(
        before.splitlines(keepends=True), after.splitlines(keepends=True),
        fromfile=f"{ref}:{path} (frozen)", tofile=f"{path} (working tree)"))
    return 1, f"FROZEN PART CHANGED in {path} -- the implementer edited its own judge:\n{diff}"


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("file", type=Path)
    parser.add_argument("--marker", default=DEFAULT_MARKER)
    parser.add_argument("--ref", default="HEAD")
    args = parser.parse_args(argv)
    code, message = compare(args.file, args.marker, args.ref)
    print(message, file=sys.stderr if code else sys.stdout)
    return code


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
