#!/usr/bin/env python3
"""Validate that a commit message follows Conventional Commits.

Usage:
    python3 scripts/check_commit_msg.py <message-file>     # git commit-msg hook
    python3 scripts/check_commit_msg.py -m "feat: ..."     # a literal message
    python3 scripts/check_commit_msg.py --range A..B       # every commit in a range (CI)

Exit 0 when valid, 1 otherwise. Used by `.githooks/commit-msg`, by CI on
every pull request, and by `scripts/guard_command.py` (which any tool can
call before running a shell command) -- one rule, one implementation.
"""
from __future__ import annotations

import argparse
import re
import subprocess
import sys

TYPES = ("feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore", "revert")
CONVENTIONAL_RE = re.compile(r"^(" + "|".join(TYPES) + r")(\([a-zA-Z0-9_.\-/]+\))?!?: .+")
# Subjects git itself generates; nobody authored them, so they aren't checked.
EXEMPT_RE = re.compile(r"^(Merge |Revert \"|fixup! |squash! )")


def subject_of(message: str) -> str:
    for line in message.splitlines():
        line = line.strip()
        if line and not line.startswith("#"):
            return line
    return ""


def problem(message: str) -> str | None:
    subject = subject_of(message)
    if not subject or EXEMPT_RE.match(subject) or CONVENTIONAL_RE.match(subject):
        return None
    return f'commit subject doesn\'t follow Conventional Commits ({"/".join(TYPES)}): "{subject}"'


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("file", nargs="?")
    parser.add_argument("-m", "--message")
    parser.add_argument("--range", dest="rev_range")
    args = parser.parse_args(argv)

    messages: list[tuple[str, str]] = []
    if args.message is not None:
        messages.append(("message", args.message))
    if args.file:
        with open(args.file, encoding="utf-8") as fh:
            messages.append((args.file, fh.read()))
    if args.rev_range:
        shas = subprocess.run(
            ["git", "rev-list", "--no-merges", args.rev_range],
            capture_output=True, text=True, check=True,
        ).stdout.split()
        for sha in shas:
            body = subprocess.run(
                ["git", "log", "-1", "--format=%B", sha], capture_output=True, text=True, check=True
            ).stdout
            messages.append((sha[:7], body))
    if not messages:
        parser.error("give a message file, -m, or --range")

    failed = False
    for origin, message in messages:
        p = problem(message)
        if p:
            print(f"{origin}: {p}", file=sys.stderr)
            failed = True
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
