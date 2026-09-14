#!/usr/bin/env python3
"""Validate internal Markdown links and anchors across the repository.

Checks, for every `[text](target)` / `![alt](target)` found in every
tracked `*.md` file (skipping fenced code blocks, since a code example that
happens to contain `[x](y)` isn't a real link):

  1. If `target` has a path component, that file actually exists on disk
     (resolved relative to the linking file's directory).
  2. If `target` has a `#fragment`, that fragment matches a real heading in
     the target file (or the current file, for a same-file `#fragment`),
     using GitHub's heading-slug algorithm.

External links (`http(s)://`, `mailto:`, `tel:`) are never checked -- this
is only about the internal cross-references this repo actually controls.

This replaces "an agent reads the diff and hopes the links still work"
with a real, deterministic check: same input always produces the same
verdict, independent of who runs it.

Usage:
    scripts/check_doc_links.py              # scan the whole repo, exit 1 if anything's broken
    scripts/check_doc_links.py path/to/file.md docs/other.md   # scan just these files
"""

from __future__ import annotations

import re
import sys
from pathlib import Path
from typing import Optional

HEADING_RE = re.compile(r"^(#{1,6})\s+(.+?)\s*$")
LINK_RE = re.compile(r"!?\[([^\]]*)\]\(([^)\s]+)(?:\s+\"[^\"]*\")?\)")
FENCE_RE = re.compile(r"^\s*```")

EXCLUDED_DIR_MARKERS = ("/.git/", "/node_modules/", "/.obsidian/")

INLINE_CODE_RE = re.compile(r"`([^`]*)`")
BOLD_RE = re.compile(r"\*\*([^*]*)\*\*")
ITALIC_RE = re.compile(r"\*([^*]*)\*")
MD_LINK_TEXT_RE = re.compile(r"\[([^\]]*)\]\([^)]*\)")


def strip_markdown(text: str) -> str:
    text = INLINE_CODE_RE.sub(r"\1", text)
    text = MD_LINK_TEXT_RE.sub(r"\1", text)
    text = BOLD_RE.sub(r"\1", text)
    text = ITALIC_RE.sub(r"\1", text)
    return text


def github_slugify(heading: str) -> str:
    text = strip_markdown(heading).strip().lower()
    text = re.sub(r"[^\w\s-]", "", text)
    text = re.sub(r"\s+", "-", text)
    return text


def is_excluded(path: Path) -> bool:
    posix = "/" + path.as_posix()
    return any(marker in posix for marker in EXCLUDED_DIR_MARKERS)


def find_repo_markdown_files(root: Path) -> list[Path]:
    return sorted(p for p in root.rglob("*.md") if not is_excluded(p))


def iter_non_fenced_lines(text: str):
    in_fence = False
    for line_no, line in enumerate(text.splitlines(), start=1):
        if FENCE_RE.match(line):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        yield line_no, line


_slug_cache: dict[Path, set[str]] = {}


def slugs_for(path: Path) -> Optional[set[str]]:
    if path in _slug_cache:
        return _slug_cache[path]
    try:
        text = path.read_text(encoding="utf-8")
    except (FileNotFoundError, UnicodeDecodeError):
        return None

    slugs: set[str] = set()
    seen: dict[str, int] = {}
    for _, line in iter_non_fenced_lines(text):
        match = HEADING_RE.match(line)
        if not match:
            continue
        base = github_slugify(match.group(2))
        n = seen.get(base, 0)
        slugs.add(base if n == 0 else f"{base}-{n}")
        seen[base] = n + 1

    _slug_cache[path] = slugs
    return slugs


def is_external(target: str) -> bool:
    return bool(re.match(r"^[a-zA-Z][a-zA-Z0-9+.\-]*:", target)) and not target.startswith("#")


def check_file(path: Path) -> list[str]:
    problems = []
    try:
        text = path.read_text(encoding="utf-8")
    except (FileNotFoundError, UnicodeDecodeError) as exc:
        return [f"{path}: could not read ({exc})"]

    for line_no, line in iter_non_fenced_lines(text):
        for match in LINK_RE.finditer(line):
            target = match.group(2)
            if is_external(target) or target.startswith("mailto:"):
                continue

            if "#" in target:
                path_part, fragment = target.split("#", 1)
            else:
                path_part, fragment = target, None

            if path_part:
                target_file = (path.parent / path_part).resolve()
                if not target_file.exists():
                    problems.append(f"{path}:{line_no}: broken link target `{target}` -- {target_file} does not exist")
                    continue
            else:
                target_file = path

            if fragment:
                slugs = slugs_for(target_file)
                if slugs is None:
                    problems.append(f"{path}:{line_no}: can't check anchor `#{fragment}` -- {target_file} isn't readable")
                elif fragment.lower() not in slugs:
                    problems.append(
                        f"{path}:{line_no}: anchor `#{fragment}` not found in {target_file} "
                        f"(no heading slugs to `{fragment.lower()}`)"
                    )

    return problems


def main(argv: Optional[list[str]] = None) -> int:
    argv = sys.argv[1:] if argv is None else argv
    files = [Path(a) for a in argv] if argv else find_repo_markdown_files(Path("."))

    all_problems: list[str] = []
    for f in files:
        all_problems.extend(check_file(f))

    if all_problems:
        for problem in all_problems:
            print(problem, file=sys.stderr)
        print(f"\n{len(all_problems)} broken internal link(s)/anchor(s).", file=sys.stderr)
        return 1

    print(f"OK -- checked {len(files)} file(s), no broken internal links or anchors.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
