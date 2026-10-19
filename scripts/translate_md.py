#!/usr/bin/env python3
"""Draft a pt-BR translation of a Markdown file with the local `translate` model.

Usage:
    python3 scripts/translate_md.py <en.md>              # prints the draft
    python3 scripts/translate_md.py <en.md> -o <pt.md>   # writes it

A model can't be trusted to leave structure alone (measured: told to keep
headings and SHALL/MUST in English, it still rewrote `## Purpose` and
`### Requirement:`). So the model never sees the structure at all:

  1. The file is split deterministically into structure (blank lines, code
     blocks, OpenSpec section headings, `Requirement:`/`Scenario:` prefixes,
     list/checkbox/`**WHEN**` prefixes) and prose segments.
  2. Only the prose segments go to the model, as a JSON array, with a JSON
     schema forcing exactly one translation per segment.
  3. Each translation is checked: inline `code` spans and SHALL/MUST
     keywords must survive unchanged. A failing segment is retried once,
     then the run fails rather than writing a bad draft.
  4. The structure is put back around the translations.

The result is a *draft*: check_bilingual_parity.py verifies its structure,
and a person still reads the prose before it's committed (AGENTS.md,
"Choosing who does the work"). Exit 3 = no local model available.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import local_llm  # noqa: E402
from check_bilingual_parity import OPENSPEC_FIXED_HEADINGS, OPENSPEC_NAMED_PREFIXES  # noqa: E402

FENCE_RE = re.compile(r"^\s*```")
HEADING_RE = re.compile(r"^(#{1,6}\s+)(.*)$")
# Leading Markdown syntax kept verbatim: quote, bullets, numbered items,
# task checkboxes with an OpenSpec number, bold scenario keywords, `**Label:**`.
PREFIX_RE = re.compile(
    r"^(\s*(?:>\s*)?(?:[-*+]\s+|\d+\.\s+)?(?:\[[ xX]\]\s+)?(?:\d+(?:\.\d+)+\s+)?"
    r"(?:\*\*(?:WHEN|THEN|AND|GIVEN)\*\*\s+)?)(.*)$"
)
CODE_SPAN_RE = re.compile(r"`[^`]+`")
KEYWORD_RE = re.compile(r"\b(?:SHALL NOT|SHALL|MUST NOT|MUST)\b")
SYSTEM = (
    "You translate software documentation from English to Brazilian Portuguese (pt-BR). "
    "You get a JSON object {\"segments\": [...]} and return {\"t\": [...]} with exactly one "
    "translation per segment, in the same order. Inside each segment keep unchanged: text in "
    "backticks, file paths, identifiers, URLs, Markdown links' targets, and the uppercase words "
    "SHALL, MUST, MUST NOT, SHALL NOT. Keep the project's English domain terms (stem, Command, "
    "Event, loop, crossfade, playback names) as they are. Translate everything else naturally."
)


@dataclass
class Piece:
    prefix: str  # kept verbatim
    text: str = ""  # translated when `translate` is True
    translate: bool = False


def split(markdown: str) -> list[Piece]:
    pieces: list[Piece] = []
    in_fence = False
    for line in markdown.split("\n"):
        if FENCE_RE.match(line):
            in_fence = not in_fence
            pieces.append(Piece(line))
            continue
        if in_fence or not line.strip():
            pieces.append(Piece(line))
            continue
        heading = HEADING_RE.match(line)
        if heading:
            marks, title = heading.groups()
            named = next((p for p in OPENSPEC_NAMED_PREFIXES if title.startswith(p)), None)
            if named:
                pieces.append(Piece(f"{marks}{named} ", title[len(named):].strip(), True))
            elif title in OPENSPEC_FIXED_HEADINGS:
                pieces.append(Piece(line))
            else:
                num = re.match(r"^(\d+\.\s+)(.*)$", title)
                if num:
                    pieces.append(Piece(marks + num.group(1), num.group(2), True))
                else:
                    pieces.append(Piece(marks, title, True))
            continue
        prefix, rest = PREFIX_RE.match(line).groups()
        pieces.append(Piece(prefix, rest, bool(rest.strip())) if rest.strip() else Piece(line))
    return pieces


def join(pieces: list[Piece], translations: list[str]) -> str:
    it = iter(translations)
    return "\n".join(p.prefix + (next(it) if p.translate else "") for p in pieces)


def segment_problem(source: str, translated: str) -> str | None:
    missing = [c for c in CODE_SPAN_RE.findall(source) if c not in translated]
    if missing:
        return f"code span(s) changed: {missing}"
    if KEYWORD_RE.findall(source) != KEYWORD_RE.findall(translated):
        return "SHALL/MUST keywords changed"
    if "\n" in translated.strip():
        return "segment split across lines"
    return None


def ask(url: str, role: dict, segments: list[str]) -> list[str]:
    payload = local_llm.build_request(role, json.dumps({"segments": segments}, ensure_ascii=False), SYSTEM)
    payload["format"] = {
        "type": "object",
        "properties": {"t": {"type": "array", "items": {"type": "string"},
                             "minItems": len(segments), "maxItems": len(segments)}},
        "required": ["t"],
    }
    payload["options"] = {"temperature": 0}
    reply = local_llm.call(url, "/api/chat", payload)
    out = json.loads(local_llm.strip_thinking(reply["message"]["content"]))["t"]
    if len(out) != len(segments):
        raise ValueError(f"model returned {len(out)} translations for {len(segments)} segments")
    return [s.strip() for s in out]


def translate(markdown: str, url: str, role: dict, batch: int = 20) -> str:
    pieces = split(markdown)
    sources = [p.text for p in pieces if p.translate]
    result: list[str] = []
    for start in range(0, len(sources), batch):
        chunk = sources[start:start + batch]
        out = ask(url, role, chunk)
        for i, (src, dst) in enumerate(zip(chunk, out)):
            if segment_problem(src, dst):
                dst = ask(url, role, [src])[0]
                problem = segment_problem(src, dst)
                if problem:
                    raise ValueError(f"segment {start + i + 1} ({src[:60]!r}): {problem}")
            result.append(dst)
    return join(pieces, result)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("source", type=Path)
    parser.add_argument("-o", "--output", type=Path)
    args = parser.parse_args(argv)

    url = local_llm.server_url()
    if url is None:
        print("No local model server configured (STEMLOFT_OLLAMA_URL).", file=sys.stderr)
        return local_llm.EXIT_UNAVAILABLE
    role = local_llm.load_roles()["translate"]
    try:
        text = translate(args.source.read_text(encoding="utf-8"), url, role)
    except (OSError, ValueError, KeyError) as exc:
        print(f"translation failed: {exc}", file=sys.stderr)
        return 1
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(text, encoding="utf-8")
    else:
        sys.stdout.write(text)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
