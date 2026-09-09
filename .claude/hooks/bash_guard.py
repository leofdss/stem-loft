#!/usr/bin/env python3
"""PreToolUse hook (matcher: Bash).

Bloqueia dois padroes que violam regras documentadas em docs/doc.md:

1. `npm install/i/add <pacote>` com argumento posicional -- o projeto proibe
   qualquer pacote npm de terceiros na camada Angular (ver
   docs/doc.md#stack-e-plataforma). `npm install`/`npm ci` sem pacote
   (lockfile ja existente) continuam liberados.
2. `git commit` cuja mensagem nao siga Conventional Commits.

Saida: JSON com hookSpecificOutput.permissionDecision "deny" bloqueia a
ferramenta antes de rodar; nenhuma saida (exit 0) libera.
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
            "Este projeto (Angular) nao usa nenhum pacote npm de terceiros "
            "alem do que o proprio Angular ja traz -- ver "
            "docs/doc.md#stack-e-plataforma. Se a instalacao for realmente "
            "necessaria, confirme com o usuario antes (fora deste hook)."
        )

    if "git commit" in cmd:
        msg = extract_commit_message(cmd)
        if msg and not CONVENTIONAL_RE.match(msg):
            return deny(
                "Mensagem de commit nao segue Conventional Commits "
                "(feat/fix/docs/style/refactor/perf/test/build/ci/chore/revert"
                f'): "{msg}"'
            )

    return 0


if __name__ == "__main__":
    sys.exit(main())
