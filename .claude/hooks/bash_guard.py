#!/usr/bin/env python3
"""Claude Code adapter (PreToolUse, matcher Bash).

Thin on purpose: the rules live in scripts/guard_command.py (AGENTS.md,
"Agent-agnostic rule"). This file only translates Claude Code's hook JSON
into a command string and a violation into a "deny" decision.
"""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scripts"))
from guard_command import violation  # noqa: E402


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0
    cmd = (payload.get("tool_input") or {}).get("command") or ""
    reason = violation(cmd) if cmd else None
    if reason:
        print(json.dumps({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": reason,
            }
        }))
    return 0


if __name__ == "__main__":
    sys.exit(main())
