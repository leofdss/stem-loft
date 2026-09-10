#!/usr/bin/env python3
"""PreToolUse hook (matcher: Bash).

Blocks two patterns that violate rules documented in docs/architecture.md:

1. `npm install/i/add <package>` with a positional argument -- the project
   forbids any third-party npm package in the Angular layer (see
   docs/architecture.md#stack-and-platform). `npm install`/`npm ci` with no
   package (an existing lockfile) stay allowed.
2. `git commit` whose message doesn't follow Conventional Commits.

Output: JSON with hookSpecificOutput.permissionDecision "deny" blocks the
tool before it runs; no output (exit 0) allows it.
"""
import json
import re
import sys

NPM_INSTALL_RE = re.compile(
    r"(^|[;&|]\s*|&&\s*)npm\s+(install|i|add)\s+(?!-)(?!$)\S"
)

HEREDOC_RE = re.compile(
    r"<<[ \t]*[\"']?(\w+)[\"']?\n(.*?)\n[ \t]*\1\b", re.DOTALL
)

INLINE_M_RE = re.compile(r"""-m\s+["']([^"']+)["']""")

CONVENTIONAL_RE = re.compile(
    r"^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)"
    r"(\([a-zA-Z0-9_.\-/]+\))?!?: .+"
)


def deny(reason: str) -> int:
    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason,
        }
    }))
    return 0


def extract_commit_message(cmd: str) -> str | None:
    heredoc_match = HEREDOC_RE.search(cmd)
    if heredoc_match:
        body = heredoc_match.group(2)
        for line in body.splitlines():
            line = line.strip()
            if line:
                return line
    inline_match = INLINE_M_RE.search(cmd)
    if inline_match:
        return inline_match.group(1).strip()
    return None


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0

    cmd = (payload.get("tool_input") or {}).get("command") or ""
    if not cmd:
        return 0

    if NPM_INSTALL_RE.search(cmd):
        return deny(
            "This project (Angular) doesn't use any third-party npm "
            "package beyond what Angular itself already brings -- see "
            "docs/architecture.md#stack-and-platform. If the install is "
            "really necessary, confirm with the user first (outside this "
            "hook)."
        )

    if "git commit" in cmd:
        msg = extract_commit_message(cmd)
        if msg and not CONVENTIONAL_RE.match(msg):
            return deny(
                "Commit message doesn't follow Conventional Commits "
                "(feat/fix/docs/style/refactor/perf/test/build/ci/chore/revert"
                f'): "{msg}"'
            )

    return 0


if __name__ == "__main__":
    sys.exit(main())
