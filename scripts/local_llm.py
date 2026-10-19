#!/usr/bin/env python3
"""Send one prompt to the project's local-model server (Ollama) by role.

Tool-neutral on purpose: a human, a shell script, or any AI agent acting as
orchestrator can hand a well-checked chore to a local model the same way,
and the role -> model mapping lives in one file
(distrobox/remote/ollama/models.toml), never in a caller.

Usage:
    echo "prompt" | python3 scripts/local_llm.py --role quick
    python3 scripts/local_llm.py --role translate --system "..." < doc.md
    python3 scripts/local_llm.py --status        # server up? which models are pulled/loaded?
    python3 scripts/local_llm.py --list-pull     # models install.sh should pull

Server URL: $STEMLOFT_OLLAMA_URL, else distrobox/remote/host.local's
STEMLOFT_OLLAMA_URL. Neither set => exit 3 ("no local model here"), which
callers must treat like the remote build host being absent: do the work
another way, not an error. Stdlib only.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import sys
import tomllib
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MODELS = ROOT / "distrobox" / "remote" / "ollama" / "models.toml"
HOST_LOCAL = ROOT / "distrobox" / "remote" / "host.local"
EXIT_UNAVAILABLE = 3


def load_roles(path: Path = MODELS) -> dict[str, dict]:
    return tomllib.loads(path.read_text(encoding="utf-8"))["roles"]


def server_url(env: dict[str, str] | None = None, host_local: Path = HOST_LOCAL) -> str | None:
    env = os.environ if env is None else env
    if env.get("STEMLOFT_OLLAMA_URL"):
        return env["STEMLOFT_OLLAMA_URL"].rstrip("/")
    if host_local.is_file():
        match = re.search(r"STEMLOFT_OLLAMA_URL=['\"]?([^'\"\s]+)", host_local.read_text(encoding="utf-8"))
        if match:
            return match.group(1).rstrip("/")
    return None


def build_request(role: dict, prompt: str, system: str | None) -> dict:
    messages = ([{"role": "system", "content": system}] if system else []) + [{"role": "user", "content": prompt}]
    return {"model": role["model"], "messages": messages, "stream": False, "think": bool(role.get("think", False))}


def strip_thinking(text: str) -> str:
    """Drop a reasoning preamble some models emit inline even with think=false
    (measured with qwen3:4b, whose default tag is a thinking-only variant)."""
    return re.sub(r"^.*?</think>\s*", "", text, flags=re.DOTALL) if "</think>" in text else text


def call(url: str, path: str, payload: dict | None = None, timeout: float = 900) -> dict:
    data = json.dumps(payload).encode() if payload is not None else None
    req = urllib.request.Request(url + path, data=data, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return json.loads(resp.read())


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--role", help="role from models.toml (quick, code, translate, heavy, ...)")
    parser.add_argument("--system", help="system prompt")
    parser.add_argument("--status", action="store_true")
    parser.add_argument("--list-pull", action="store_true")
    args = parser.parse_args(argv)

    roles = load_roles()
    if args.list_pull:
        print("\n".join(r["model"] for r in roles.values() if r.get("pull")))
        return 0

    url = server_url()
    if url is None:
        print("No local model server configured (STEMLOFT_OLLAMA_URL).", file=sys.stderr)
        return EXIT_UNAVAILABLE

    try:
        if args.status:
            version = call(url, "/api/version", timeout=5)["version"]
            pulled = {m["name"] for m in call(url, "/api/tags", timeout=5)["models"]}
            loaded = {m["name"]: m.get("size_vram", 0) for m in call(url, "/api/ps", timeout=5)["models"]}
            print(f"ollama {version} at {url}")
            for name, role in roles.items():
                model = role["model"]
                state = "loaded" if model in loaded else ("pulled" if model in pulled else "missing")
                print(f"  {name:<10} {model:<20} {state}")
            return 0

        if not args.role or args.role not in roles:
            parser.error(f"--role must be one of: {', '.join(roles)}")
        prompt = sys.stdin.read()
        reply = call(url, "/api/chat", build_request(roles[args.role], prompt, args.system))
    except (urllib.error.URLError, TimeoutError, ConnectionError) as exc:
        print(f"Local model server at {url} unreachable: {exc}", file=sys.stderr)
        return EXIT_UNAVAILABLE

    sys.stdout.write(strip_thinking(reply["message"]["content"]).strip() + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
