#!/usr/bin/env python3
"""PostToolUse hook (matcher: Write|Edit).

Runs scripts/check_npm_deps.py whenever a `package.json` is written.
Closes the gap bash_guard.py's `npm install <pkg>` block doesn't cover: a
dependency added by hand-editing package.json and then running a bare
`npm install`/`npm ci` (no positional package -- which bash_guard
deliberately allows, since that's just "install what the lockfile already
says").

Silent (exit 0) if the edited file isn't a package.json, if the checker is
missing, or if the dependencies still match the approved baseline.
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

    if not file_path.replace("\\", "/").endswith("package.json"):
        return 0

    root = repo_root()
    if root is None:
        return 0

    checker = root / "scripts" / "check_npm_deps.py"
    if not checker.is_file():
        return 0

    result = subprocess.run(
        [sys.executable, str(checker), file_path],
        cwd=root,
        capture_output=True,
        text=True,
        timeout=15,
    )

    if result.returncode != 0:
        sys.stderr.write(result.stdout)
        sys.stderr.write(result.stderr)
        return 2

    return 0


if __name__ == "__main__":
    sys.exit(main())
