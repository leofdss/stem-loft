#!/usr/bin/env python3
"""Check structural parity between each `foo.md` / `foo.pt-BR.md` pair.

This can't verify translation *quality* -- that's still a judgment call for
whoever reviews the diff. What it CAN verify mechanically, and what
actually catches most real drift, is:

  1. Both files have the same sequence of heading levels (e.g.
     `[1, 2, 2, 3, 2]`) -- a whole section added/removed in only one
     language shows up as a length or shape mismatch, independent of what
     the headings say.
  2. Fenced code blocks whose language is one that should never be
     translated (bash/sh/shell/json/toml/yaml/yml/ini) are byte-identical
     between the two files, in the same order. `mermaid` is deliberately
     excluded -- a diagram's node labels are prose and may legitimately be
     translated (verified against this repo: architecture.md's Mermaid
     diagram differs from its pt-BR sibling in exactly the labels, nothing
     else).

Pairs are discovered automatically (same foo.md <-> foo.pt-BR.md
convention as pt_br_sync_reminder.py), not hardcoded, so a new translated
doc is picked up without touching this script.

Usage:
    scripts/check_bilingual_parity.py                 # scan the whole repo
    scripts/check_bilingual_parity.py TODO.md          # check just this pair
"""

from __future__ import annotations

import re
import sys
from pathlib import Path
from typing import Optional

PT_BR_SUFFIX = ".pt-BR.md"
FENCE_OPEN_RE = re.compile(r"^\s*```\s*([A-Za-z0-9_+-]*)\s*$")
FENCE_CLOSE_RE = re.compile(r"^\s*```\s*$")
HEADING_RE = re.compile(r"^(#{1,6})\s+\S")

# Languages where the content itself (commands, config) must be identical
# across languages, as opposed to prose or diagrams that may be translated.
INVARIANT_LANGS = {"bash", "sh", "shell", "json", "toml", "yaml", "yml", "ini"}

EXCLUDED_DIR_MARKERS = ("/.git/", "/node_modules/", "/.obsidian/")


def is_excluded(path: Path) -> bool:
    posix = "/" + path.as_posix()
    return any(marker in posix for marker in EXCLUDED_DIR_MARKERS)


def counterpart_path(path: Path) -> Optional[Path]:
    name = path.name
    if name.endswith(PT_BR_SUFFIX):
        return path.with_name(name[: -len(PT_BR_SUFFIX)] + ".md")
    if name.endswith(".md"):
        return path.with_name(name[: -len(".md")] + PT_BR_SUFFIX)
    return None


def discover_pairs(root: Path) -> list[tuple[Path, Path]]:
    pairs = []
    for path in sorted(root.rglob("*.md")):
        if is_excluded(path) or path.name.endswith(PT_BR_SUFFIX):
            continue
        pt_path = counterpart_path(path)
        if pt_path is not None and pt_path.is_file():
            pairs.append((path, pt_path))
    return pairs


def heading_shape(text: str) -> list[int]:
    shape = []
    in_fence = False
    for line in text.splitlines():
        if FENCE_OPEN_RE.match(line) or FENCE_CLOSE_RE.match(line):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        match = HEADING_RE.match(line)
        if match:
            shape.append(len(re.match(r"^#+", line).group(0)))
    return shape


def code_blocks(text: str) -> list[tuple[str, str]]:
    blocks = []
    lang = None
    buffer: list[str] = []
    in_fence = False
    for line in text.splitlines():
        if not in_fence:
            open_match = FENCE_OPEN_RE.match(line)
            if open_match:
                in_fence = True
                lang = open_match.group(1).lower()
                buffer = []
            continue
        if FENCE_CLOSE_RE.match(line):
            in_fence = False
            blocks.append((lang or "", "\n".join(buffer)))
            continue
        buffer.append(line)
    return blocks


def normalize_code(code: str) -> str:
    """Strip the two kinds of difference that are expected, not drift.

    - The `.pt-BR` language marker itself, so a command's own filename
      argument (`TODO.pt-BR.md`, `docs/kanban.pt-BR.md`, ...) compares
      equal to its EN counterpart (`TODO.md`, `docs/kanban.md`, ...).
    - Trailing `# comment` text on a line, which this repo's existing docs
      already translate (verified against distrobox/remote/README.md's
      `# one-shot push...` vs `# envio unico...`) -- only the command
      itself has to match, not its inline commentary.
    """
    code = code.replace(".pt-BR", "")
    lines = [line.split(" #", 1)[0].rstrip() for line in code.splitlines()]
    return "\n".join(lines).strip()


def compare_pair(en_path: Path, pt_path: Path) -> list[str]:
    problems = []
    en_text = en_path.read_text(encoding="utf-8")
    pt_text = pt_path.read_text(encoding="utf-8")

    en_shape = heading_shape(en_text)
    pt_shape = heading_shape(pt_text)
    if en_shape != pt_shape:
        problems.append(
            f"{en_path} vs {pt_path}: heading structure differs -- "
            f"EN levels {en_shape} vs PT-BR levels {pt_shape}"
        )

    en_blocks = [(lang, content) for lang, content in code_blocks(en_text) if lang in INVARIANT_LANGS]
    pt_blocks = [(lang, content) for lang, content in code_blocks(pt_text) if lang in INVARIANT_LANGS]

    if len(en_blocks) != len(pt_blocks):
        problems.append(
            f"{en_path} vs {pt_path}: different number of {sorted(INVARIANT_LANGS)} code blocks "
            f"({len(en_blocks)} vs {len(pt_blocks)})"
        )
    else:
        for index, ((en_lang, en_code), (pt_lang, pt_code)) in enumerate(zip(en_blocks, pt_blocks)):
            if normalize_code(en_code) != normalize_code(pt_code):
                problems.append(
                    f"{en_path} vs {pt_path}: ```{en_lang} code block #{index + 1} differs between languages "
                    "(commands/config must be identical, not translated)"
                )

    return problems


def main(argv: Optional[list[str]] = None) -> int:
    argv = sys.argv[1:] if argv is None else argv

    if argv:
        pairs = []
        for arg in argv:
            path = Path(arg)
            if path.name.endswith(PT_BR_SUFFIX):
                en_path, pt_path = counterpart_path(path), path
            else:
                en_path, pt_path = path, counterpart_path(path)
            if en_path is None or pt_path is None or not en_path.is_file() or not pt_path.is_file():
                print(f"error: no complete pt-BR/EN pair for {path}", file=sys.stderr)
                return 2
            pairs.append((en_path, pt_path))
    else:
        pairs = discover_pairs(Path("."))

    all_problems: list[str] = []
    for en_path, pt_path in pairs:
        all_problems.extend(compare_pair(en_path, pt_path))

    if all_problems:
        for problem in all_problems:
            print(problem, file=sys.stderr)
        print(f"\n{len(all_problems)} bilingual parity issue(s) across {len(pairs)} pair(s).", file=sys.stderr)
        return 1

    print(f"OK -- {len(pairs)} bilingual pair(s) structurally in sync.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
