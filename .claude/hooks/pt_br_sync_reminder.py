#!/usr/bin/env python3
"""PostToolUse hook (matcher: Write|Edit).

Several docs have a Brazilian Portuguese sibling: foo.md <-> foo.pt-BR.md.
This hook fires whenever a file matching either half of such a pair is
written and -- only if the counterpart file actually exists -- runs
scripts/check_bilingual_parity.py on that specific pair.

That script can't judge translation quality, but it does mechanically
verify the two things that most commonly drift silently: the heading
structure (a section added/removed in only one language) and any fenced
bash/json/toml/yaml code block (commands and config must be identical
across languages, never translated). If it finds nothing, this hook stays
silent -- no more "please go check" on every single edit regardless of
whether anything could plausibly be wrong. If the checker itself can't run
for some reason, this falls back to the old generic reminder so a real
problem never goes unflagged just because the tool broke.

Generic by design: it derives the counterpart path from the filename
itself (foo.md <-> foo.pt-BR.md) rather than a hardcoded list, so new
translated doc pairs are picked up automatically.
"""
import json
import os
import subprocess
import sys
from pathlib import Path

PT_BR_SUFFIX = ".pt-BR.md"


def counterpart_path(path: str) -> str:
    if path.endswith(PT_BR_SUFFIX):
        return path[: -len(PT_BR_SUFFIX)] + ".md"
    if path.endswith(".md"):
        return path[: -len(".md")] + PT_BR_SUFFIX
    return ""


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


def generic_reminder(path: str, counterpart: str) -> None:
    sys.stderr.write(
        f"{os.path.basename(counterpart)} is the pt-BR/EN sibling of "
        f"{os.path.basename(path)} -- check it still matches after this edit.\n"
    )


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0

    tool_input = payload.get("tool_input") or {}
    tool_response = payload.get("tool_response") or {}
    file_path = tool_input.get("file_path") or tool_response.get("filePath") or ""

    if not file_path:
        return 0

    path = file_path.replace("\\", "/")
    counterpart = counterpart_path(path)

    if not counterpart or not os.path.isfile(counterpart):
        return 0

    root = repo_root()
    checker = root / "scripts" / "check_bilingual_parity.py" if root else None

    if checker is None or not checker.is_file():
        generic_reminder(path, counterpart)
        return 2

    result = subprocess.run(
        [sys.executable, str(checker), path],
        cwd=root,
        capture_output=True,
        text=True,
        timeout=30,
    )

    if result.returncode == 0:
        return 0

    if result.returncode == 1 and result.stderr:
        # Real, specific mismatches found -- pass them through as-is.
        sys.stderr.write(result.stderr)
        return 2

    # Checker itself errored (returncode 2, crash, timeout, ...) -- fall back
    # rather than silently skip the reminder.
    generic_reminder(path, counterpart)
    return 2


if __name__ == "__main__":
    sys.exit(main())
