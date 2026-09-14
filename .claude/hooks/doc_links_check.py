#!/usr/bin/env python3
"""PostToolUse hook (matcher: Write|Edit).

Runs scripts/check_doc_links.py across the whole repo whenever a `.md` file
is written. Unlike readme_sync_reminder.py (which just says "go check"),
this is a real, deterministic check: it only speaks up when it actually
found a broken internal link or a `#anchor` that doesn't match any real
heading, and says exactly which one and where -- no agent judgment
involved in detecting the problem, only in fixing it.

Silent (exit 0) if the edited file isn't Markdown, if the check script is
missing, or if everything checks out clean.
"""
import json
import subprocess
import sys
from pathlib import Path


def repo_root() -> Path | None:
    try:
        out = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            capture_output=True, text=True, timeout=5,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    if out.returncode != 0:
        return None
    return Path(out.stdout.strip())


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0

    tool_input = payload.get("tool_input") or {}
    tool_response = payload.get("tool_response") or {}
    file_path = tool_input.get("file_path") or tool_response.get("filePath") or ""

    if not file_path.endswith(".md"):
        return 0

    root = repo_root()
    if root is None:
        return 0

    checker = root / "scripts" / "check_doc_links.py"
    if not checker.is_file():
        return 0

    result = subprocess.run(
        [sys.executable, str(checker)],
        cwd=root,
        capture_output=True,
        text=True,
        timeout=30,
    )

    if result.returncode != 0:
        sys.stderr.write(result.stdout)
        sys.stderr.write(result.stderr)
        return 2

    return 0


if __name__ == "__main__":
    sys.exit(main())
