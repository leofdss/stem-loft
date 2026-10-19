#!/usr/bin/env python3
"""Enforce AGENTS.md's "Agent-agnostic rule" mechanically.

The rule: every piece of project knowledge or enforcement lives in a
tool-neutral place (AGENTS.md, docs/, openspec/, .agents/skills/,
scripts/, .githooks/, CI). A tool-specific location (.claude/, .opencode/,
CLAUDE.md, ...) may only hold a thin adapter that points back to it.

What this checks:
  1. Instruction files for a specific tool (CLAUDE.md, GEMINI.md,
     .github/copilot-instructions.md, ...) reference AGENTS.md and stay
     short -- a pointer, not a second copy of the rules.
  2. `.claude/skills`, if present, is a symlink to `.agents/skills`.
  3. Every Markdown file inside a tool directory (agents, commands, ...)
     references an existing `AGENTS.md` or `.agents/skills/<name>/SKILL.md`.
  4. Every hook script inside a tool directory calls into `scripts/` and
     stays short -- logic belongs in scripts/, the adapter only translates.
  5. Canonical playbooks (.agents/skills/*/SKILL.md) don't depend on one
     tool's mechanics (hook event names, tool-only function names).

Usage: python3 scripts/check_agent_agnostic.py     (exit 1 on any finding)
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

TOOL_INSTRUCTION_FILES = ("CLAUDE.md", "GEMINI.md", ".github/copilot-instructions.md", ".cursorrules", ".windsurfrules")
TOOL_DIRS = (".claude", ".opencode", ".cursor", ".gemini", ".codex", ".windsurf", ".continue", ".roo", ".kiro")
MAX_INSTRUCTION_LINES = 40
MAX_HOOK_LINES = 60
HOOK_SUFFIXES = (".py", ".sh", ".js", ".ts", ".mjs")
TOOL_ONLY_TERMS = re.compile(
    r"\b(PostToolUse|PreToolUse|ReportFindings|AskUserQuestion|TodoWrite|subagent_type|tool_input)\b"
)
CANONICAL_REF_RE = re.compile(r"(AGENTS\.md|\.agents/skills/[\w.-]+/SKILL\.md)")


def code_lines(text: str) -> int:
    return sum(1 for line in text.splitlines() if line.strip() and not line.strip().startswith("#"))


def check(root: Path) -> list[str]:
    problems: list[str] = []

    for name in TOOL_INSTRUCTION_FILES:
        path = root / name
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8")
        if "AGENTS.md" not in text:
            problems.append(f"{name}: must point to AGENTS.md (the tool-neutral instructions)")
        if len(text.splitlines()) > MAX_INSTRUCTION_LINES:
            problems.append(
                f"{name}: {len(text.splitlines())} lines -- keep tool instruction files under "
                f"{MAX_INSTRUCTION_LINES}; move the content to AGENTS.md or docs/"
            )

    skills_link = root / ".claude" / "skills"
    if skills_link.exists() or skills_link.is_symlink():
        if not skills_link.is_symlink() or skills_link.resolve() != (root / ".agents" / "skills").resolve():
            problems.append(".claude/skills: must be a symlink to ../.agents/skills, not a copy")

    for tool_dir in TOOL_DIRS:
        base = root / tool_dir
        if not base.is_dir():
            continue
        for path in sorted(base.rglob("*")):
            if path.is_symlink() or not path.is_file():
                continue
            rel = path.relative_to(root).as_posix()
            text = path.read_text(encoding="utf-8", errors="replace")
            if path.suffix == ".md":
                refs = CANONICAL_REF_RE.findall(text)
                if not refs:
                    problems.append(f"{rel}: must reference AGENTS.md or a .agents/skills/<name>/SKILL.md it adapts")
                for ref in refs:
                    if not (root / ref).is_file():
                        problems.append(f"{rel}: references {ref}, which doesn't exist")
            elif path.suffix in HOOK_SUFFIXES:
                if "scripts" not in text:
                    problems.append(f"{rel}: hook adapters must delegate to scripts/, not hold the logic")
                if code_lines(text) > MAX_HOOK_LINES:
                    problems.append(
                        f"{rel}: {code_lines(text)} code lines -- adapters stay under {MAX_HOOK_LINES}; "
                        "move the logic into scripts/"
                    )

    for skill in sorted((root / ".agents" / "skills").glob("*/SKILL.md")):
        for n, line in enumerate(skill.read_text(encoding="utf-8").splitlines(), 1):
            match = TOOL_ONLY_TERMS.search(line)
            if match:
                problems.append(
                    f"{skill.relative_to(root)}:{n}: tool-specific term {match.group(1)!r} in a canonical playbook"
                )
    return problems


def main() -> int:
    problems = check(ROOT)
    for p in problems:
        print(p, file=sys.stderr)
    if problems:
        print(f"\n{len(problems)} agent-agnostic violation(s) -- see AGENTS.md, \"Agent-agnostic rule\".", file=sys.stderr)
        return 1
    print("OK -- tool-specific files are thin adapters over tool-neutral sources.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
