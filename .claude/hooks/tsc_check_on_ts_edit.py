#!/usr/bin/env python3
"""PostToolUse hook (matcher: Write|Edit).

Same convention as cargo_check_on_rust_edit.py, for the Angular side: if
the edited file is `.ts`/`.html` under a directory with a `tsconfig.json`
above it, runs `tsc --noEmit` there and hands the compiler's errors back.

This is what makes it safe to ever delegate an Angular task to a cheaper
model: without it, a type error only surfaces when a human happens to run
`ng build` -- with it, the mistake comes back on the very next edit, the
same self-correction loop Rust already gets from cargo_check.

Silent (exit 0, no output) if there's no tsconfig.json above the file, if
`node_modules` hasn't been installed yet (dependencies not fetched), or if
the file isn't `.ts`/`.html`.

On a compile failure, exits with status 2 and prints tsc's errors to
stderr.
"""
import json
import os
import subprocess
import sys


def find_tsconfig_dir(start_dir: str) -> str | None:
    directory = os.path.abspath(start_dir)
    root = os.path.abspath(os.sep)
    while True:
        if os.path.isfile(os.path.join(directory, "tsconfig.json")):
            return directory
        if directory == root:
            return None
        directory = os.path.dirname(directory)


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0

    tool_input = payload.get("tool_input") or {}
    tool_response = payload.get("tool_response") or {}
    file_path = tool_input.get("file_path") or tool_response.get("filePath") or ""

    if not (file_path.endswith(".ts") or file_path.endswith(".html")):
        return 0

    project_dir = find_tsconfig_dir(os.path.dirname(os.path.abspath(file_path)))
    if project_dir is None:
        return 0

    tsc_bin = os.path.join(project_dir, "node_modules", ".bin", "tsc")
    if not os.path.isfile(tsc_bin):
        # Dependencies never installed (npm ci/install not run yet) -- same
        # "the project hasn't started yet" no-op as cargo_check when there's
        # no Cargo.toml, not an error.
        return 0

    result = subprocess.run(
        [tsc_bin, "--noEmit", "-p", "tsconfig.app.json"]
        if os.path.isfile(os.path.join(project_dir, "tsconfig.app.json"))
        else [tsc_bin, "--noEmit"],
        cwd=project_dir,
        capture_output=True,
        text=True,
    )

    if result.returncode != 0:
        sys.stderr.write(f"tsc --noEmit failed in {project_dir}:\n")
        sys.stderr.write(result.stdout)
        sys.stderr.write(result.stderr)
        return 2

    return 0


if __name__ == "__main__":
    sys.exit(main())
