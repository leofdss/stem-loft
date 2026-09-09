#!/usr/bin/env python3
"""PostToolUse hook (matcher: Write|Edit).

Se o arquivo editado for .rs e existir um Cargo.toml em algum diretorio acima
dele, roda `cargo check` nesse crate. Silencioso (exit 0, sem output) se nao
houver Cargo.toml (projeto Rust ainda nao iniciado), se o arquivo nao for
.rs, ou se `cargo` nao estiver instalado.

Em falha de compilacao, sai com status 2 e imprime os erros do cargo em
stderr -- isso volta como feedback ao agente, para autocorrecao imediata sem
precisar que o usuario rode `cargo build` manualmente.
"""
import json
import os
import shutil
import subprocess
import sys


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0

    tool_input = payload.get("tool_input") or {}
    tool_response = payload.get("tool_response") or {}
    file_path = tool_input.get("file_path") or tool_response.get("filePath") or ""

    if not file_path.endswith(".rs"):
        return 0

    directory = os.path.dirname(os.path.abspath(file_path))
    root = os.path.abspath(os.sep)
    while directory != root and not os.path.isfile(os.path.join(directory, "Cargo.toml")):
        directory = os.path.dirname(directory)

    if not os.path.isfile(os.path.join(directory, "Cargo.toml")):
        return 0

    if shutil.which("cargo") is None:
        return 0

    result = subprocess.run(
        ["cargo", "check", "--quiet", "--message-format=short"],
        cwd=directory,
        capture_output=True,
        text=True,
    )

    if result.returncode != 0:
        sys.stderr.write(f"cargo check falhou em {directory}:\n")
        sys.stderr.write(result.stdout)
        sys.stderr.write(result.stderr)
        return 2

    return 0


if __name__ == "__main__":
    sys.exit(main())
