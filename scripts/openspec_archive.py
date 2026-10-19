#!/usr/bin/env python3
"""Archive a finished OpenSpec change in both languages.

Usage:
    python3 scripts/openspec_archive.py <change-id>

OpenSpec writes artifacts in one language per project
(https://github.com/Fission-AI/OpenSpec/blob/main/docs/multi-language.md),
and `openspec archive` only merges `specs/<cap>/spec.md` deltas into
`openspec/specs/<cap>/spec.md`. This project keeps a pt-BR copy of every
artifact (see openspec/README.md), so archiving has to merge the pt-BR
delta (`i18n/pt-BR/specs/<cap>/spec.md`) into the pt-BR main spec
(`openspec/specs/<cap>/spec.pt-BR.md`) too.

Rather than re-implementing OpenSpec's merge rules, this runs the same
pinned CLI twice:
  1. on a throwaway "shadow" tree where the pt-BR files sit at the paths
     OpenSpec expects, then copies the merged result back as spec.pt-BR.md;
  2. on the real tree (`openspec archive -y`), which merges the English
     delta and moves the whole change folder -- pt-BR files included --
     under openspec/changes/archive/.
Finally the change's entry is dropped from openspec/board.toml (archived
changes are "Done" by definition).
"""
from __future__ import annotations

import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OPENSPEC = ROOT / "openspec"
CLI = ROOT / "scripts" / "openspec.sh"
PT = "pt-BR"


def run_cli(args: list[str], root: Path | None = None) -> None:
    env = dict(os.environ)
    if root is not None:
        env["OPENSPEC_ROOT"] = str(root)
    subprocess.run([str(CLI), *args], check=True, env=env)


def build_shadow(change: str, shadow: Path) -> list[str]:
    """Lay the pt-BR artifacts out where OpenSpec expects them. Returns the
    capabilities the pt-BR delta touches."""
    src = OPENSPEC / "changes" / change
    dst = shadow / "openspec" / "changes" / change
    dst.mkdir(parents=True)
    shutil.copy(OPENSPEC / "config.yaml", shadow / "openspec" / "config.yaml")
    shutil.copy(src / ".openspec.yaml", dst / ".openspec.yaml")
    for name in ("proposal", "tasks", "design"):
        translated = src / f"{name}.{PT}.md"
        if translated.is_file():
            shutil.copy(translated, dst / f"{name}.md")
        elif (src / f"{name}.md").is_file():
            shutil.copy(src / f"{name}.md", dst / f"{name}.md")
    caps: list[str] = []
    pt_specs = src / "i18n" / PT / "specs"
    if pt_specs.is_dir():
        shutil.copytree(pt_specs, dst / "specs")
        for spec in sorted(pt_specs.rglob("spec.md")):
            cap = spec.parent.relative_to(pt_specs).as_posix()
            caps.append(cap)
            main_pt = OPENSPEC / "specs" / cap / f"spec.{PT}.md"
            if main_pt.is_file():
                target = shadow / "openspec" / "specs" / cap / "spec.md"
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy(main_pt, target)
    return caps


def drop_board_entry(text: str, change: str) -> str:
    """Remove the `[change]` table (header through the line before the next
    table or EOF) from board.toml's text."""
    pattern = re.compile(rf"^\[{re.escape(change)}\]\n(?:(?!\[).*\n?)*", re.MULTILINE)
    return pattern.sub("", text).rstrip("\n") + "\n"


def main(argv: list[str]) -> int:
    if len(argv) != 1:
        print(__doc__, file=sys.stderr)
        return 2
    change = argv[0]
    if not (OPENSPEC / "changes" / change).is_dir():
        print(f"no active change named {change!r} under openspec/changes/", file=sys.stderr)
        return 1

    run_cli(["validate", change, "--strict"])

    with tempfile.TemporaryDirectory() as tmp:
        shadow = Path(tmp)
        caps = build_shadow(change, shadow)
        if caps:
            run_cli(["archive", change, "-y"], root=shadow)
            for cap in caps:
                merged = shadow / "openspec" / "specs" / cap / "spec.md"
                out = OPENSPEC / "specs" / cap / f"spec.{PT}.md"
                out.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy(merged, out)

    run_cli(["archive", change, "-y"])

    board = OPENSPEC / "board.toml"
    if board.is_file():
        board.write_text(drop_board_entry(board.read_text(encoding="utf-8"), change), encoding="utf-8")
    print(f"Archived {change} (en + {PT}). Regenerate the board: python3 scripts/openspec_to_kanban.py")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
