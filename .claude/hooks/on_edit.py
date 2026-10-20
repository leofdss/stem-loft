#!/usr/bin/env python3
"""Claude Code adapter (PostToolUse, matcher Write|Edit).

Thin on purpose: all logic lives in scripts/check_changed.py, which any
human, git hook, or other AI tool runs the same way (AGENTS.md,
"Agent-agnostic rule"). This file only translates Claude Code's hook JSON
into a path and the result back into exit status 2 + stderr.
"""
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0
    tool_input = payload.get("tool_input") or {}
    tool_response = payload.get("tool_response") or {}
    path = tool_input.get("file_path") or tool_response.get("filePath")
    if not path:
        return 0
    result = subprocess.run(
        [sys.executable, str(ROOT / "scripts" / "check_changed.py"), "--json", path],
        cwd=ROOT, capture_output=True, text=True, timeout=280,
    )
    try:
        findings = json.loads(result.stdout)
    except ValueError:
        return 0
    lines = [f"ERROR: {e}" for e in findings["errors"]] + [f"note: {n}" for n in findings["notes"]]
    if not lines:
        return 0
    sys.stderr.write("\n".join(lines) + "\n")
    return 2


if __name__ == "__main__":
    sys.exit(main())
