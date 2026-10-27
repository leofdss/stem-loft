#!/usr/bin/env python3
"""Project policy for shell commands, checked *before* they run.

Usage:
    python3 scripts/guard_command.py "<shell command>"

Exit 0 (silent) when the command is allowed; exit 1 with the reason on
stderr when it breaks a project rule. Meant for any tool that can run a
check before executing a shell command (`.claude/hooks/bash_guard.py` is
the Claude Code adapter); humans get the same rules from AGENTS.md, and
`.githooks/commit-msg` enforces the commit rule for everyone.

Rules:
  1. `npm install/i/add <package>` with a positional argument -- no
     third-party npm package beyond the approved baseline (see
     docs/architecture.md#stack-and-platform). `npm install`/`npm ci` with
     no package (from the lockfile) stay allowed; `scripts/check_npm_deps.py`
     catches a dependency added by hand-editing package.json instead.
  2. A git commit whose message doesn't follow Conventional Commits (same
     rule as `scripts/check_commit_msg.py`).
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from check_commit_msg import problem as commit_problem  # noqa: E402

NPM_INSTALL_RE = re.compile(
    r"(^|[;&|]\s*|&&\s*)npm\s+(install|i|add)\s+"
    r"(?!-)(?!$)"
    r"(?!\d*[<>])"  # not a redirection (`2>&1`, `>out.txt`, `2>/tmp/log`, ...)
    r"(?![|&;])"    # not a shell control operator (`|`, `&`, `&>`, `;`, ...)
    r"\S"
)
# Only an actual `git commit` invocation, not the words inside a heredoc body.
GIT_COMMIT_RE = re.compile(r"(^|[;&|(]\s*|&&\s*)git(\s+-C\s+\S+)?\s+commit\b", re.MULTILINE)
HEREDOC_RE = re.compile(r"<<[ \t]*[\"']?(\w+)[\"']?\n(.*?)\n[ \t]*\1\b", re.DOTALL)
INLINE_M_RE = re.compile(r"""-m\s+["']([^"']+)["']""")


def extract_commit_message(cmd: str) -> str | None:
    """The message of the `git commit` in `cmd`: a heredoc opened on the
    commit's own line (`git commit -F- <<EOF`, `-m "$(cat <<EOF`), else the
    first `-m` after the commit. Any other heredoc in the same command (a
    script fed to python3 before committing) is not the message."""
    for heredoc in HEREDOC_RE.finditer(cmd):
        line_start = cmd.rfind("\n", 0, heredoc.start()) + 1
        if GIT_COMMIT_RE.search(cmd[line_start:heredoc.start()]):
            return heredoc.group(2)
    stripped = HEREDOC_RE.sub("", cmd)
    commit = GIT_COMMIT_RE.search(stripped)
    inline = INLINE_M_RE.search(stripped, commit.end()) if commit else None
    return inline.group(1) if inline else None


def violation(cmd: str) -> str | None:
    if NPM_INSTALL_RE.search(cmd):
        return (
            "This project (Angular) doesn't use any third-party npm package "
            "beyond what Angular itself already brings -- see "
            "docs/architecture.md#stack-and-platform. If the install is really "
            "necessary, get the maintainer's approval first."
        )
    if GIT_COMMIT_RE.search(HEREDOC_RE.sub("", cmd)):
        msg = extract_commit_message(cmd)
        if msg:
            return commit_problem(msg)
    return None


def main(argv: list[str]) -> int:
    if len(argv) != 1:
        print(__doc__, file=sys.stderr)
        return 2
    reason = violation(argv[0])
    if reason:
        print(reason, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
