#!/usr/bin/env python3
"""Flag any dependency in ui/package.json beyond the approved baseline.

architecture.md#stack-and-platform's policy is that the Angular layer uses
no third-party npm package beyond what Angular itself already brings.
bash_guard.py enforces the common way that gets violated (`npm install
<pkg>`), but it can't see a package added by hand-editing package.json and
then running a bare `npm install`/`npm ci` (no positional package, which
bash_guard deliberately allows since that's just "install what the
lockfile already says"). This script closes that gap: it's a plain diff
against a hardcoded baseline, not a judgment call.

ALLOWED is intentionally a flat, hardcoded list, not something computed --
it should almost never change, and a PR that adds to it is exactly the
kind of thing that deserves a human's explicit sign-off, not a script
auto-approving whatever happens to be in node_modules today.

Usage:
    scripts/check_npm_deps.py [path/to/package.json]   # default: ui/package.json
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Optional

# Snapshot of ui/package.json's dependencies + devDependencies as of the
# Angular CLI scaffold (see README.md's "Dev environment" / architecture.md's
# "Stack and platform"). Angular's own packages (@angular/*), the two
# runtime libraries Angular itself requires (rxjs, tslib), and the
# build/lint/test tooling already wired into `npm run build`/`test`.
ALLOWED = {
    "@angular/common",
    "@angular/compiler",
    "@angular/core",
    "@angular/forms",
    "@angular/platform-browser",
    "@angular/router",
    "rxjs",
    "tslib",
    "@angular/build",
    "@angular/cli",
    "@angular/compiler-cli",
    "jsdom",
    "prettier",
    "typescript",
    "vitest",
    # Coverage provider for `ng test`'s coverage reporting -- approved by
    # the user 2026-09-14 alongside enabling coverage in angular.json (see
    # CLAUDE.md's testing-policy section).
    "@vitest/coverage-v8",
}

DEFAULT_PACKAGE_JSON = Path("ui/package.json")


def check(package_json: Path) -> list[str]:
    try:
        data = json.loads(package_json.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        return [f"{package_json}: could not read/parse ({exc})"]

    declared = set(data.get("dependencies", {})) | set(data.get("devDependencies", {}))
    unexpected = sorted(declared - ALLOWED)

    if not unexpected:
        return []

    return [
        f"{package_json}: unexpected dependency `{name}` -- not in the approved baseline "
        "(architecture.md#stack-and-platform: no third-party npm package beyond what Angular "
        "itself brings). If this was intentionally approved by the user, add it to ALLOWED in "
        "scripts/check_npm_deps.py in the same change."
        for name in unexpected
    ]


def main(argv: Optional[list[str]] = None) -> int:
    argv = sys.argv[1:] if argv is None else argv
    package_json = Path(argv[0]) if argv else DEFAULT_PACKAGE_JSON

    if not package_json.is_file():
        print(f"OK -- {package_json} doesn't exist, nothing to check.")
        return 0

    problems = check(package_json)
    if problems:
        for problem in problems:
            print(problem, file=sys.stderr)
        return 1

    print(f"OK -- {package_json}'s dependencies match the approved baseline.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
