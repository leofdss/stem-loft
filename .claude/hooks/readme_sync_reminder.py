#!/usr/bin/env python3
"""PostToolUse hook (matcher: Write|Edit).

README.md is meant to summarize a handful of other documents: TODO.md,
docs/architecture.md, docs/project-name.md, the Distrobox dev-environment
docs, and the agents/skills under .claude/. Nothing enforces that summary
stays accurate as those documents change, so this hook nudges the agent to
check whenever one of them is edited.

If the edited file is one of those source-of-truth documents (and isn't
README.md itself), prints a one-line reminder to stderr and exits with
status 2 -- the same convention cargo_check_on_rust_edit.py uses to hand
feedback back to the agent, repurposed here as a reminder rather than a
build failure. Silent (exit 0, no output) for every other file, including
README.md itself (editing it directly needs no reminder to update it).
"""
import json
import sys

WATCHED_SUFFIXES = (
    "/TODO.md",
    "/docs/architecture.md",
    "/docs/project-name.md",
    "/distrobox/README.md",
    "/distrobox/Containerfile",
)

WATCHED_DIR_MARKERS = (
    "/.claude/agents/",
    "/.claude/skills/",
)


def is_root_readme(path: str) -> bool:
    return path == "README.md" or (
        path.endswith("/README.md") and not path.endswith("/distrobox/README.md")
    )


def is_watched(path: str) -> bool:
    if any(path.endswith(suffix) for suffix in WATCHED_SUFFIXES):
        return True
    return any(marker in path for marker in WATCHED_DIR_MARKERS)


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

    if is_root_readme(path) or not is_watched(path):
        return 0

    sys.stderr.write(
        f"README.md may need a matching update after this change to {file_path} -- "
        "check status/feature/link accuracy.\n"
    )
    return 2


if __name__ == "__main__":
    sys.exit(main())
