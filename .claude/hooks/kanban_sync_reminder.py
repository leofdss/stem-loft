#!/usr/bin/env python3
"""PostToolUse hook (matcher: Write|Edit).

docs/kanban.html is a generated artifact -- a pure function of TODO.md and
TODO.pt-BR.md, produced by `scripts/todo_to_kanban.py --html`. Nothing
enforces that it actually gets regenerated after one of those two files
changes, so this hook fires whenever either is written and reminds the
agent to run the generator -- the same reminder-not-build-failure
convention as readme_sync_reminder.py / pt_br_sync_reminder.py.

This is a nudge, not the guarantee: the guarantee is that
scripts/todo_to_kanban.py --html is a deterministic function of the two
TODO files (same input always produces the same output, regardless of who
runs it or why), so re-running it is always correct and never needs
interpretation. This hook just makes sure it actually gets re-run.
"""
import json
import sys

WATCHED_SUFFIXES = ("/TODO.md", "/TODO.pt-BR.md")
WATCHED_NAMES = ("TODO.md", "TODO.pt-BR.md")


def is_watched(path: str) -> bool:
    return path in WATCHED_NAMES or any(path.endswith(suffix) for suffix in WATCHED_SUFFIXES)


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

    if not is_watched(path):
        return 0

    sys.stderr.write(
        "docs/kanban.html is generated from TODO.md + TODO.pt-BR.md -- run "
        "`python3 scripts/todo_to_kanban.py --html` to regenerate it before "
        "considering this change done.\n"
    )
    return 2


if __name__ == "__main__":
    sys.exit(main())
