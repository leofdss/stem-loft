#!/usr/bin/env python3
"""Let a local model implement one file against a check it can't change.

Usage (from the repo root):
    python3 scripts/llm_task.py --role code \\
        --task openspec/changes/<id>/llm-task.md \\
        --edit src-tauri/src/core/persistence.rs --freeze-from '#[cfg(test)]' \\
        --context docs/architecture.md \\
        --check 'cd src-tauri && cargo test -q --lib core::persistence' \\
        --max-iter 6 --log /tmp/llm-run

How it works -- the "harness" around the model:
  1. The edited file is split at --freeze-from: everything before it is the
     model's section; everything from it on (e.g. the `#[cfg(test)]` module)
     is FROZEN. The model sees the frozen part read-only and can only
     return its own section, so it can't weaken the tests that judge it
     (AGENTS.md: never let one model write the code *and* its test).
  2. Each iteration is stateless: task + context + frozen part + the
     current section + the last check's output. The prompt doesn't grow
     with history, so it fits the role's context window every time. Both
     parts are shown with their line numbers in the file, and every
     location the check output names (`--> file:137`, `panicked at
     file:547`) comes back as an excerpt marked YOUR SECTION or FROZEN.
  3. The model's reply must contain one fenced code block: the complete new
     section. It's written into the file and --check runs.
  4. Green check => done. Red => its output (errors first, trimmed) is the
     feedback for the next iteration, up to --max-iter. If the model sends
     back the same code, or the check fails exactly as before, the next
     prompt says so (STALLED) and the temperature rises to break the loop.
  5. On failure the file is restored (unless --keep), so a bad run never
     leaves the tree half-edited.

Every iteration's prompt, reply, check output and Ollama timings go to
--log (one JSON line each in runs.jsonl), so runs can be compared. Prints a
one-line JSON summary. Exit 0 green, 1 not green, 3 no local model.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import signal
import subprocess
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import local_llm  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
FENCE_RE = re.compile(r"```[a-zA-Z0-9_+-]*\n(.*?)```", re.DOTALL)
# `--> src/core/x.rs:137:48` (rustc) and `panicked at src/core/x.rs:547:9` (test panics).
LOCATION_RE = re.compile(r"(?:-->|panicked at)\s+([^\s:]+):(\d+)(?::\d+)?")
MAX_FEEDBACK_LINES = 120
EXCERPT_RADIUS = 3
MAX_EXCERPTS = 6
STALL_TEMPERATURE_STEP = 0.3
MAX_TEMPERATURE = 0.9

SYSTEM = """You are a careful senior engineer working inside an automated harness.
Each turn you get a task, read-only context, a FROZEN part of the file you may
not change, the current version of YOUR SECTION of that file, and the output of
the last check run. Reply with exactly one fenced code block containing the
COMPLETE new version of your section -- never a diff, never the frozen part,
never placeholders like "rest unchanged". Your section is placed before the
frozen part in the same file, so everything the frozen part uses must be
defined in your section. Fix what the check output reports first. Do not add
dependencies that the task doesn't allow."""


def split(text: str, marker: str | None) -> tuple[str, str]:
    if not marker:
        return text, ""
    index = text.find(marker)
    if index < 0:
        raise ValueError(f"--freeze-from marker {marker!r} not found in the edited file")
    return text[:index], text[index:]


LINE_PREFIX_RE = re.compile(r"^\s*\d+ \| ?")


def strip_line_numbers(code: str) -> str:
    """Models echo the `N | ` prefixes the prompt shows despite being told not
    to (measured: every model, every run, once lines were numbered). Strip
    them deterministically when most non-empty lines carry one."""
    lines = code.split("\n")
    non_empty = [line for line in lines if line.strip()]
    if not non_empty or sum(bool(LINE_PREFIX_RE.match(line)) for line in non_empty) < 0.8 * len(non_empty):
        return code
    return "\n".join(LINE_PREFIX_RE.sub("", line, count=1) for line in lines)


def extract_section(reply: str) -> str | None:
    blocks = FENCE_RE.findall(local_llm.strip_thinking(reply))
    if not blocks:
        return None
    return strip_line_numbers(max(blocks, key=len)).rstrip() + "\n\n"


def trim_feedback(output: str) -> str:
    """Keep what a model can act on: compiler errors and failing-test
    panics first, warnings and progress noise dropped, capped in length."""
    lines = output.splitlines()
    keep: list[str] = []
    in_warning = False
    for line in lines:
        if line.startswith("warning"):
            in_warning = True
            continue
        if line.startswith(("error", "---- ", "thread '", "failures:", "test result")):
            in_warning = False
        if in_warning and not line.strip():
            in_warning = False
            continue
        if not in_warning and not line.lstrip().startswith(("Compiling", "Finished", "Running", "Blocking")):
            keep.append(line)
    if len(keep) > MAX_FEEDBACK_LINES:
        keep = keep[:MAX_FEEDBACK_LINES] + [f"... ({len(keep) - MAX_FEEDBACK_LINES} more lines trimmed)"]
    return "\n".join(keep).strip()


def numbered(text: str, first_line: int = 1) -> str:
    """Prefix each line with its line number in the edited file, so the model
    can map a compiler's `file:137` straight onto what it sees."""
    lines = text.rstrip("\n").split("\n")
    width = len(str(first_line + len(lines) - 1))
    return "\n".join(f"{first_line + i:>{width}} | {line}" for i, line in enumerate(lines))


def excerpts(check_output: str, edit_rel: str, section: str, frozen: str) -> str:
    """For every location the check output points at inside the edited file,
    show the lines around it and say whose they are. A failing assertion in
    the FROZEN tests then arrives with its exact expectation, and a compiler
    error with the exact line the model wrote."""
    file_lines = (section + frozen).split("\n")
    section_lines = section.count("\n")
    seen: set[int] = set()
    blocks: list[str] = []
    for path, line_s in LOCATION_RE.findall(check_output):
        if not edit_rel.endswith(path.lstrip("./")) and not path.endswith(Path(edit_rel).name):
            continue
        line = int(line_s)
        if line in seen or not 1 <= line <= len(file_lines):
            continue
        seen.add(line)
        start = max(1, line - EXCERPT_RADIUS)
        end = min(len(file_lines), line + EXCERPT_RADIUS)
        owner = "your section" if line <= section_lines else (
            "FROZEN part -- the tests are right: change YOUR SECTION (signatures, behavior, "
            "exact messages) to match what this line expects"
        )
        blocks.append(f"Line {line} is in {owner}:\n" + numbered("\n".join(file_lines[start - 1:end]), start))
        if len(blocks) >= MAX_EXCERPTS:
            break
    return "\n\n".join(blocks)


def build_prompt(task: str, context: list[tuple[str, str]], edit_path: str, frozen: str,
                 section: str, feedback: str | None, iteration: int, stalled: str | None = None) -> str:
    parts = [f"# Task\n\n{task.strip()}"]
    for path, text in context:
        parts.append(f"# Read-only context: {path}\n\n{text.strip()}")
    first_frozen_line = section.count("\n") + 1
    if frozen:
        parts.append(
            f"# FROZEN part of {edit_path} (read-only, starts at line {first_frozen_line}, after your section)"
            f"\n\n```\n{numbered(frozen, first_frozen_line)}\n```"
        )
    parts.append(
        f"# YOUR SECTION of {edit_path} (current version, lines 1-{first_frozen_line - 1}; "
        f"the `N | ` prefixes are line numbers -- don't include them in your reply)"
        f"\n\n```\n{numbered(section)}\n```"
    )
    if feedback is None:
        parts.append("# Check output\n\nNot run yet -- this is the first iteration.")
    else:
        parts.append(f"# Check output after iteration {iteration - 1} (FAILED)\n\n```\n{feedback}\n```")
    if stalled:
        parts.append(f"# STALLED\n\n{stalled}")
    parts.append("Reply with one fenced code block: the complete new version of YOUR SECTION.")
    return "\n\n".join(parts)


def run_check(command: str, timeout: int) -> tuple[bool, str, float]:
    """Run the check in its own process group, so a hung test binary (not
    just the shell) is killed on timeout."""
    start = time.monotonic()
    proc = subprocess.Popen(command, shell=True, cwd=ROOT, stdout=subprocess.PIPE,
                            stderr=subprocess.STDOUT, text=True, start_new_session=True)
    try:
        out, _ = proc.communicate(timeout=timeout)
        ok = proc.returncode == 0
    except subprocess.TimeoutExpired:
        os.killpg(proc.pid, signal.SIGKILL)
        out, _ = proc.communicate()
        ok = False
        out = (out or "") + (
            f"\nThe check did not finish within {timeout}s: a test HUNG. That is almost always a "
            "deadlock (a Mutex locked again while already held -- e.g. calling a method that locks "
            "while you hold the same lock) or an infinite loop in your section."
        )
    return ok, out, time.monotonic() - start


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--role", default="code")
    ap.add_argument("--task", required=True, type=Path)
    ap.add_argument("--edit", required=True, type=Path)
    ap.add_argument("--freeze-from")
    ap.add_argument("--start-from", type=Path, help="initial content for the model's section (e.g. a skeleton)")
    ap.add_argument("--context", action="append", default=[], type=Path)
    ap.add_argument("--check", required=True)
    ap.add_argument("--check-timeout", type=int, default=180)
    ap.add_argument("--max-iter", type=int, default=6)
    ap.add_argument("--log", type=Path)
    ap.add_argument("--keep", action="store_true", help="leave the last attempt in place on failure")
    args = ap.parse_args(argv)

    url = local_llm.server_url()
    if url is None:
        print("No local model server configured (STEMLOFT_OLLAMA_URL).", file=sys.stderr)
        return local_llm.EXIT_UNAVAILABLE
    roles = local_llm.load_roles()
    role = roles[args.role]

    edit = args.edit if args.edit.is_absolute() else ROOT / args.edit
    original = edit.read_text(encoding="utf-8")
    section, frozen = split(original, args.freeze_from)
    if args.start_from:
        section = args.start_from.read_text(encoding="utf-8")
    task = args.task.read_text(encoding="utf-8")
    context = [(str(p), p.read_text(encoding="utf-8")) for p in args.context]
    if args.log:
        args.log.mkdir(parents=True, exist_ok=True)

    feedback: str | None = None
    stalled: str | None = None
    previous_feedback: str | None = None
    temperature = float(role.get("options", {}).get("temperature", 0.2))
    edit_rel = str(args.edit)
    green = False
    totals = {"iterations": 0, "model_s": 0.0, "check_s": 0.0, "prompt_tokens": 0, "output_tokens": 0}
    started = time.monotonic()
    for iteration in range(1, args.max_iter + 1):
        totals["iterations"] = iteration
        prompt = build_prompt(task, context, edit_rel, frozen, section, feedback, iteration, stalled)
        payload = local_llm.build_request(role, prompt, SYSTEM)
        payload["options"] = {**role.get("options", {}), "temperature": temperature}
        t0 = time.monotonic()
        try:
            reply = local_llm.call(url, "/api/chat", payload, timeout=1800)
        except OSError as exc:
            print(f"local model unreachable: {exc}", file=sys.stderr)
            edit.write_text(original, encoding="utf-8")
            return local_llm.EXIT_UNAVAILABLE
        model_s = time.monotonic() - t0
        totals["model_s"] += model_s
        totals["prompt_tokens"] += reply.get("prompt_eval_count", 0)
        totals["output_tokens"] += reply.get("eval_count", 0)
        content = reply["message"]["content"]

        new_section = extract_section(content)
        unchanged = new_section is not None and new_section.strip() == section.strip()
        if new_section is None:
            ok, out, check_s = False, "Your reply had no fenced code block. Reply with exactly one.", 0.0
        else:
            section = new_section
            edit.write_text(section + frozen, encoding="utf-8")
            ok, out, check_s = run_check(args.check, args.check_timeout)
        totals["check_s"] += check_s
        feedback = trim_feedback(out)
        located = excerpts(out, edit_rel, section, frozen)
        if located:
            feedback += "\n\n# Where (lines from the edited file)\n\n" + located

        # Stall detection: same code back, or the very same failure again.
        stalled = None
        if not ok and (unchanged or feedback == previous_feedback):
            temperature = min(MAX_TEMPERATURE, temperature + STALL_TEMPERATURE_STEP)
            stalled = (
                ("Your last reply was IDENTICAL to the previous version. " if unchanged else "")
                + "The check failed with exactly the same output as before, so your last change did not "
                "address it. Read the 'Where' excerpts: make the change those exact lines require, "
                "even if it means rewriting the function involved."
            )
            totals["stalls"] = totals.get("stalls", 0) + 1
        previous_feedback = feedback

        if args.log:
            with open(args.log / "runs.jsonl", "a", encoding="utf-8") as fh:
                fh.write(json.dumps({
                    "iteration": iteration, "model": role["model"], "ok": ok,
                    "temperature": payload["options"]["temperature"], "stalled": bool(stalled),
                    "model_s": round(model_s, 1), "check_s": round(check_s, 1),
                    "prompt_tokens": reply.get("prompt_eval_count"), "output_tokens": reply.get("eval_count"),
                    "prompt": prompt, "reply": content, "feedback": feedback,
                }) + "\n")
        print(f"iter {iteration}: {'GREEN' if ok else 'red'} (model {model_s:.0f}s, check {check_s:.0f}s)",
              file=sys.stderr)
        if ok:
            green = True
            break

    if not green and not args.keep:
        edit.write_text(original, encoding="utf-8")
    summary = {"green": green, "model": role["model"], "wall_s": round(time.monotonic() - started, 1),
               **{k: round(v, 1) if isinstance(v, float) else v for k, v in totals.items()}}
    print(json.dumps(summary))
    return 0 if green else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
