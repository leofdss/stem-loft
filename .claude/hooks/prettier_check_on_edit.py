#!/usr/bin/env python3
"""PostToolUse hook (matcher: Write|Edit).

Same convention as cargo_check_on_rust_edit.py / tsc_check_on_ts_edit.py:
runs `prettier --check` on a `.ts`/`.html`/`.css`/`.scss` file under `ui/`
after it's written, and hands back which file isn't formatted per
`ui/.prettierrc`. Report-only, like every other check hook here -- it
never rewrites the file itself (`prettier --write` from inside a hook
could step on the very edit that just landed).

`ng lint` isn't wired up the same way: this project has no lint builder
configured (`ng add angular-eslint` would add a new npm dependency, which
needs the user's explicit approval first, same as any other new package --
see architecture.md#stack-and-platform and check_npm_deps.py's baseline).

Silent (exit 0, no output) if the edited file isn't one Prettier formats
here, if there's no `ui/` project above it, or if `node_modules/.bin/
prettier` hasn't been installed yet.
"""
import json
import os
import subprocess
import sys

PRETTIER_EXTENSIONS = (".ts", ".html", ".css", ".scss")


def find_project_dir(start_dir: str) -> str | None:
    directory = os.path.abspath(start_dir)
    root = os.path.abspath(os.sep)
    while True:
        if os.path.isfile(os.path.join(directory, ".prettierrc")):
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

    if not file_path.endswith(PRETTIER_EXTENSIONS):
        return 0

    project_dir = find_project_dir(os.path.dirname(os.path.abspath(file_path)))
    if project_dir is None:
        return 0

    prettier_bin = os.path.join(project_dir, "node_modules", ".bin", "prettier")
    if not os.path.isfile(prettier_bin):
        return 0

    result = subprocess.run(
        [prettier_bin, "--check", os.path.abspath(file_path)],
        cwd=project_dir,
        capture_output=True,
        text=True,
    )

    if result.returncode != 0:
        rel_path = os.path.relpath(file_path, project_dir)
        sys.stderr.write(
            f"{rel_path} isn't formatted per .prettierrc -- run "
            f"`npx prettier --write {rel_path}` (from {project_dir}) before calling this done.\n"
        )
        sys.stderr.write(result.stdout)
        sys.stderr.write(result.stderr)
        return 2

    return 0


if __name__ == "__main__":
    sys.exit(main())
