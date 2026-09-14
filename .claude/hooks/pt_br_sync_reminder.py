#!/usr/bin/env python3
"""PostToolUse hook (matcher: Write|Edit).

Several docs have a Brazilian Portuguese sibling: foo.md <-> foo.pt-BR.md.
Each pair carries a manual "Maintenance note" pointing at its counterpart,
but nothing enforces that the two actually stay in sync as one of them is
edited. This hook fires whenever a file matching either half of such a
pair is written, and -- only if the counterpart file actually exists --
reminds the agent to check it too.

Generic by design: it derives the counterpart path from the filename
itself (foo.md <-> foo.pt-BR.md) rather than a hardcoded list, so new
translated doc pairs are picked up automatically. Same convention as
cargo_check_on_rust_edit.py / readme_sync_reminder.py: reminder to stderr,
exit status 2.
"""
import json
import os
import sys

PT_BR_SUFFIX = ".pt-BR.md"


def counterpart_path(path: str) -> str:
    if path.endswith(PT_BR_SUFFIX):
        return path[: -len(PT_BR_SUFFIX)] + ".md"
    if path.endswith(".md"):
        return path[: -len(".md")] + PT_BR_SUFFIX
    return ""


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

    sys.stderr.write(
        f"{os.path.basename(counterpart)} is the pt-BR/EN sibling of "
        f"{os.path.basename(path)} -- check it still matches after this edit.\n"
    )
    return 2


if __name__ == "__main__":
    sys.exit(main())
