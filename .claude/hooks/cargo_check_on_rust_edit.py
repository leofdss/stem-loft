#!/usr/bin/env python3
"""PostToolUse hook (matcher: Write|Edit).

If the edited file is .rs and a Cargo.toml exists in some directory above
it, runs `cargo check` in that crate. Silent (exit 0, no output) if there's
no Cargo.toml (the Rust project hasn't started yet), if the file isn't .rs,
or if `cargo` isn't installed.

On a compile failure, exits with status 2 and prints cargo's errors to
stderr -- this comes back as feedback to the agent, for immediate
self-correction without needing the user to run `cargo build` manually.
"""
import json
import os
import shutil
import subprocess
import sys


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0

    tool_input = payload.get("tool_input") or {}
    tool_response = payload.get("tool_response") or {}
    file_path = tool_input.get("file_path") or tool_response.get("filePath") or ""

    if not file_path.endswith(".rs"):
        return 0

    directory = os.path.dirname(os.path.abspath(file_path))
    root = os.path.abspath(os.sep)
    while directory != root and not os.path.isfile(os.path.join(directory, "Cargo.toml")):
        directory = os.path.dirname(directory)

    if not os.path.isfile(os.path.join(directory, "Cargo.toml")):
        return 0

    if shutil.which("cargo") is None:
        return 0

    result = subprocess.run(
        ["cargo", "check", "--quiet", "--message-format=short"],
        cwd=directory,
        capture_output=True,
        text=True,
    )

    if result.returncode != 0:
        sys.stderr.write(f"cargo check failed in {directory}:\n")
        sys.stderr.write(result.stdout)
        sys.stderr.write(result.stderr)
        return 2

    return 0


if __name__ == "__main__":
    sys.exit(main())
