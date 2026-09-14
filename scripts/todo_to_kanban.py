#!/usr/bin/env python3
"""Convert TODO.md's task board into a Mermaid `kanban` diagram, or into the
full standalone `docs/kanban.html` viewer.

TODO.md (and its TODO.pt-BR.md sibling) structure the v1 implementation
backlog as:

    ## Backlog | In Progress | Review | Done
    ### TASK-NNN — Title
    - **Branch:** `feat/some-branch`
    ...

This script reads that structure and can emit either:

  1. The raw Mermaid `kanban` block for one board file (default mode) --
     nothing else in the file (intro prose, "How to use this board", the
     Icebox section) is touched, since only `##` headings matching one of
     the four known column names are treated as columns.
  2. The complete `docs/kanban.html` page (`--html` mode) -- reads TODO.md
     *and* its TODO.pt-BR.md counterpart, computes the per-column stats,
     and writes the whole self-contained file (styles, the Mermaid CDN
     script tag, both language boards, the EN/PT toggle). Nothing in this
     mode is hand-assembled by whoever runs it -- the HTML is a pure
     function of the two TODO files, so the output is identical no matter
     who (human or agent) runs the command.

Any AI agent editing TODO.md or TODO.pt-BR.md's task/column structure MUST
re-run `--html` mode afterwards -- see the root CLAUDE.md and the
`kanban_sync_reminder.py` hook, which nudges this automatically.

Usage:
    scripts/todo_to_kanban.py                    # reads TODO.md, prints the Mermaid block to stdout
    scripts/todo_to_kanban.py TODO.pt-BR.md
    scripts/todo_to_kanban.py -o docs/kanban.md   # write the Mermaid block to a file instead

    scripts/todo_to_kanban.py --html              # regenerate docs/kanban.html from TODO.md + TODO.pt-BR.md
    scripts/todo_to_kanban.py --html -o out.html  # write the full page elsewhere
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import Optional

COLUMN_RE = re.compile(r"^##\s+(.+?)\s*$")
TASK_RE = re.compile(r"^###\s+(\S+)\s+—\s+(.+?)\s*$")
BRANCH_RE = re.compile(r"\*\*[^:*]*:\*\*\s*`([^`]+)`")

# Fixes the column order in the diagram regardless of the order tasks
# happen to appear in the source file.
KNOWN_COLUMNS = ("Backlog", "In Progress", "Review", "Done")

PT_BR_SUFFIX = ".pt-BR.md"

# Pinned exact version, per project policy of not trusting a floating "latest".
# 10.9.x and earlier do NOT ship the `kanban` diagram type at all (verified by
# grepping the minified bundle) -- if you bump this, verify the new version's
# cdnjs bundle still contains the string "kanban" before shipping it, since a
# missing diagram type fails silently at render time with a cryptic Mermaid
# parse error, not a load error.
MERMAID_VERSION = "11.15.0"

Task = dict[str, Optional[str]]
Columns = dict[str, list[Task]]


def counterpart_path(path: Path) -> Optional[Path]:
    """foo.md <-> foo.pt-BR.md, same convention as pt_br_sync_reminder.py."""
    name = path.name
    if name.endswith(PT_BR_SUFFIX):
        return path.with_name(name[: -len(PT_BR_SUFFIX)] + ".md")
    if name.endswith(".md"):
        return path.with_name(name[: -len(".md")] + PT_BR_SUFFIX)
    return None


def parse_board(text: str) -> Columns:
    columns: Columns = {name: [] for name in KNOWN_COLUMNS}
    current_column: Optional[str] = None
    current_task: Optional[Task] = None

    for line in text.splitlines():
        column_match = COLUMN_RE.match(line)
        if column_match:
            heading = column_match.group(1)
            current_column = heading if heading in KNOWN_COLUMNS else None
            current_task = None
            continue

        if current_column is None:
            continue

        task_match = TASK_RE.match(line)
        if task_match:
            current_task = {
                "id": task_match.group(1),
                "title": task_match.group(2),
                "branch": None,
            }
            columns[current_column].append(current_task)
            continue

        stripped = line.lstrip()
        if current_task is not None and current_task["branch"] is None and stripped.startswith("- **Branch"):
            branch_match = BRANCH_RE.search(line)
            if branch_match:
                current_task["branch"] = branch_match.group(1)

    return columns


def slugify(text: str) -> str:
    slug = re.sub(r"[^A-Za-z0-9]+", "_", text).strip("_")
    return slug or "col"


def mermaid_label(text: str) -> str:
    # Mermaid delimits labels with ", so a literal " in a title would break
    # the diagram's syntax -- swap it for a close single quote instead.
    return text.replace('"', "'")


def to_mermaid(columns: Columns) -> str:
    lines = ["kanban"]
    for column_name in KNOWN_COLUMNS:
        col_id = slugify(column_name)
        lines.append(f'  {col_id}["{mermaid_label(column_name)}"]')
        for task in columns[column_name]:
            task_id = slugify(str(task["id"]))
            label = f'{task["id"]} — {task["title"]}'
            if task["branch"]:
                label += f' ({task["branch"]})'
            lines.append(f'    {task_id}["{mermaid_label(label)}"]')
    return "\n".join(lines) + "\n"


def html_escape(text: str) -> str:
    return text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def task_count_hint(columns: Columns) -> str:
    all_ids = [str(task["id"]) for tasks in columns.values() for task in tasks]
    if not all_ids:
        return "no tasks yet"

    def numeric_part(task_id: str) -> str:
        match = re.search(r"(\d+)$", task_id)
        return match.group(1) if match else task_id

    ordered = sorted(all_ids, key=numeric_part)
    count = len(ordered)
    noun = "task" if count == 1 else "tasks"
    if ordered[0] == ordered[-1]:
        return f"{count} {noun}, {ordered[0]}"
    return f"{count} {noun}, {ordered[0]} – {ordered[-1]}"


HTML_TEMPLATE = """<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>StemLoft Kanban</title>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@400;500;600;700&family=IBM+Plex+Mono:wght@400;500;600&display=swap">
<style>
  :root {{
    color-scheme: light dark;
    --bg: #f5f3ee;
    --surface: #ffffff;
    --surface-alt: #ece9e1;
    --border: #dcd7cc;
    --text: #201f1c;
    --muted: #6b6659;
    --accent: #2f6e68;
    --warn: #b6772e;
    --info: #3b6fa0;
    --done: #3f8f5f;
    --shadow: 0 1px 2px rgba(32, 31, 28, 0.06), 0 6px 16px rgba(32, 31, 28, 0.05);
  }}
  @media (prefers-color-scheme: dark) {{
    :root {{
      --bg: #14171a;
      --surface: #1b1f23;
      --surface-alt: #20252b;
      --border: #2b3137;
      --text: #e9e6df;
      --muted: #9b978e;
      --accent: #5fc2b9;
      --warn: #e2a25a;
      --info: #85b1de;
      --done: #7fd39a;
      --shadow: 0 1px 2px rgba(0, 0, 0, 0.3), 0 8px 20px rgba(0, 0, 0, 0.35);
    }}
  }}

  * {{ box-sizing: border-box; }}
  body {{
    margin: 0;
    background: var(--bg);
    color: var(--text);
    font-family: "IBM Plex Sans", system-ui, -apple-system, "Segoe UI", sans-serif;
    padding-inline: 20px;
  }}
  main {{
    max-width: 1180px;
    margin: 0 auto;
    padding-block: 40px 56px;
    display: flex;
    flex-direction: column;
    gap: 28px;
  }}

  code {{ font-family: "IBM Plex Mono", ui-monospace, monospace; }}

  header.page {{
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    align-items: flex-end;
    gap: 20px;
    border-bottom: 1px solid var(--border);
    padding-bottom: 20px;
  }}
  .titleblock {{ max-width: 62ch; }}
  .eyebrow {{
    font-family: "IBM Plex Mono", monospace;
    font-size: 0.72rem;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--accent);
    margin: 0 0 8px;
  }}
  h1 {{
    margin: 0 0 8px;
    font-size: clamp(1.6rem, 1.2rem + 1.4vw, 2.15rem);
    font-weight: 700;
    letter-spacing: -0.01em;
    text-wrap: balance;
  }}
  p.lede {{
    margin: 0;
    color: var(--muted);
    font-size: 0.98rem;
    line-height: 1.55;
  }}
  p.lede code {{ color: var(--text); background: var(--surface-alt); padding: 0.05em 0.4em; border-radius: 4px; font-size: 0.88em; }}

  .lang-toggle {{
    display: inline-flex;
    background: var(--surface-alt);
    border: 1px solid var(--border);
    border-radius: 999px;
    padding: 3px;
    gap: 2px;
    flex-shrink: 0;
  }}
  .lang-toggle button {{
    font-family: "IBM Plex Mono", monospace;
    font-size: 0.78rem;
    font-weight: 500;
    letter-spacing: 0.02em;
    border: none;
    background: transparent;
    color: var(--muted);
    padding: 7px 14px;
    border-radius: 999px;
    cursor: pointer;
    transition: background 0.15s ease, color 0.15s ease;
  }}
  .lang-toggle button:hover {{ color: var(--text); }}
  .lang-toggle button[aria-pressed="true"] {{
    background: var(--surface);
    color: var(--text);
    box-shadow: var(--shadow);
  }}
  .lang-toggle button:focus-visible {{
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }}

  .stats {{
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 12px;
  }}
  .stat {{
    background: var(--surface);
    border: 1px solid var(--border);
    border-left: 3px solid var(--stat-color, var(--border));
    border-radius: 10px;
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    box-shadow: var(--shadow);
  }}
  .stat .n {{
    font-family: "IBM Plex Mono", monospace;
    font-variant-numeric: tabular-nums;
    font-size: 1.6rem;
    font-weight: 600;
    line-height: 1;
  }}
  .stat .label {{ font-size: 0.8rem; color: var(--muted); }}
  .stat.backlog {{ --stat-color: var(--muted); }}
  .stat.progress {{ --stat-color: var(--warn); }}
  .stat.review {{ --stat-color: var(--info); }}
  .stat.done {{ --stat-color: var(--done); }}

  .board-panel {{
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 14px;
    box-shadow: var(--shadow);
    overflow: hidden;
  }}
  .board-panel .panel-head {{
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    padding: 14px 20px;
    border-bottom: 1px solid var(--border);
    background: var(--surface-alt);
  }}
  .panel-head .file {{ font-family: "IBM Plex Mono", monospace; font-size: 0.82rem; color: var(--muted); }}
  .panel-head .file strong {{ color: var(--text); font-weight: 600; }}
  .panel-head .cmd-hint {{ font-size: 0.78rem; color: var(--muted); }}
  .board-scroll {{ overflow-x: auto; padding: 22px 20px 26px; min-height: 120px; }}
  .board-scroll svg {{ max-width: none !important; }}
  #board-pt-wrap[hidden], #board-en-wrap[hidden] {{ display: none; }}
  .loading {{ color: var(--muted); font-size: 0.9rem; }}

  .legend {{
    display: flex;
    flex-wrap: wrap;
    gap: 14px;
    padding: 0 20px 20px;
    font-size: 0.8rem;
    color: var(--muted);
  }}
  .legend span {{ display: inline-flex; align-items: center; gap: 6px; }}
  .legend .dot {{ width: 9px; height: 9px; border-radius: 50%; display: inline-block; }}
  .legend .dot.backlog {{ background: var(--muted); }}
  .legend .dot.progress {{ background: var(--warn); }}
  .legend .dot.review {{ background: var(--info); }}
  .legend .dot.done {{ background: var(--done); }}

  footer.notes {{
    display: flex;
    flex-wrap: wrap;
    gap: 24px;
    justify-content: space-between;
    align-items: flex-start;
    border-top: 1px solid var(--border);
    padding-top: 18px;
    font-size: 0.85rem;
    color: var(--muted);
  }}
  footer.notes .cmd {{
    background: var(--surface-alt);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 10px 14px;
    font-family: "IBM Plex Mono", monospace;
    font-size: 0.8rem;
    color: var(--text);
    max-width: 100%;
    overflow-x: auto;
    white-space: pre;
  }}
  footer.notes .col {{ max-width: 46ch; }}
  footer.notes .col b {{ color: var(--text); }}

  @media (max-width: 620px) {{
    .stats {{ grid-template-columns: repeat(2, 1fr); }}
    header.page {{ align-items: flex-start; }}
  }}
</style>
</head>
<body>
<main>
  <header class="page">
    <div class="titleblock">
      <p class="eyebrow">TODO.md &rarr; Mermaid kanban</p>
      <h1>StemLoft implementation board</h1>
      <p class="lede">
        Generated by <code>scripts/todo_to_kanban.py --html</code> from the
        current <code>##</code>/<code>###</code> structure of
        <code>TODO.md</code> and <code>TODO.pt-BR.md</code> -- this file is
        a pure function of those two, never hand-edited. Open it straight
        from disk, no server needed.
      </p>
    </div>
    <div class="lang-toggle" role="group" aria-label="Board language">
      <button type="button" id="btn-en" aria-pressed="true">EN &middot; TODO.md</button>
      <button type="button" id="btn-pt" aria-pressed="false">PT-BR &middot; TODO.pt-BR.md</button>
    </div>
  </header>

  <section class="stats" aria-label="Task counts per column">
    <div class="stat backlog"><span class="n">{count_backlog}</span><span class="label">Backlog</span></div>
    <div class="stat progress"><span class="n">{count_progress}</span><span class="label">In Progress</span></div>
    <div class="stat review"><span class="n">{count_review}</span><span class="label">Review</span></div>
    <div class="stat done"><span class="n">{count_done}</span><span class="label">Done</span></div>
  </section>

  <section class="board-panel">
    <div class="panel-head">
      <span class="file">Source: <strong id="source-file">TODO.md</strong></span>
      <span class="cmd-hint">{task_count_hint}</span>
    </div>
    <div class="board-scroll">
      <p class="loading" id="loading-note">Rendering board&hellip;</p>
      <div id="board-en-wrap">
        <pre class="mermaid" id="board-en">{mermaid_en}</pre>
      </div>
      <div id="board-pt-wrap" hidden>
        <pre class="mermaid" id="board-pt">{mermaid_pt}</pre>
      </div>
    </div>
    <div class="legend" aria-hidden="true">
      <span><span class="dot backlog"></span>Backlog &mdash; not started</span>
      <span><span class="dot progress"></span>In Progress</span>
      <span><span class="dot review"></span>Review</span>
      <span><span class="dot done"></span>Done</span>
    </div>
  </section>

  <footer class="notes">
    <div class="col">
      <b>Keep this in sync</b> &mdash; whenever <code>TODO.md</code> or
      <code>TODO.pt-BR.md</code>'s task/column structure changes, regenerate
      this file (the <code>kanban_sync_reminder.py</code> hook nudges this
      automatically, but don't rely on remembering to check the reminder).
    </div>
    <div class="col">
      <b>Regenerate</b>
      <div class="cmd">python3 scripts/todo_to_kanban.py --html</div>
    </div>
  </footer>
</main>

<script src="https://cdnjs.cloudflare.com/ajax/libs/mermaid/{mermaid_version}/mermaid.min.js"></script>
<script>
  (function () {{
    var isDark = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
    mermaid.initialize({{ startOnLoad: false, theme: isDark ? 'dark' : 'default' }});

    mermaid.run({{ querySelector: '.mermaid' }}).then(function () {{
      var note = document.getElementById('loading-note');
      if (note) note.remove();
    }}).catch(function (err) {{
      var note = document.getElementById('loading-note');
      if (note) note.textContent = 'Could not render the diagram (' + err.message + '). Mermaid failed to load -- check your internet connection.';
    }});

    var btnEn = document.getElementById('btn-en');
    var btnPt = document.getElementById('btn-pt');
    var wrapEn = document.getElementById('board-en-wrap');
    var wrapPt = document.getElementById('board-pt-wrap');
    var sourceFile = document.getElementById('source-file');

    function showEn() {{
      wrapEn.hidden = false;
      wrapPt.hidden = true;
      btnEn.setAttribute('aria-pressed', 'true');
      btnPt.setAttribute('aria-pressed', 'false');
      sourceFile.textContent = 'TODO.md';
    }}
    function showPt() {{
      wrapEn.hidden = true;
      wrapPt.hidden = false;
      btnEn.setAttribute('aria-pressed', 'false');
      btnPt.setAttribute('aria-pressed', 'true');
      sourceFile.textContent = 'TODO.pt-BR.md';
    }}
    btnEn.addEventListener('click', showEn);
    btnPt.addEventListener('click', showPt);
  }})();
</script>
</body>
</html>
"""


def build_html(columns_en: Columns, columns_pt: Columns) -> str:
    ids_en = {str(task["id"]) for tasks in columns_en.values() for task in tasks}
    ids_pt = {str(task["id"]) for tasks in columns_pt.values() for task in tasks}
    if ids_en != ids_pt:
        only_en = sorted(ids_en - ids_pt)
        only_pt = sorted(ids_pt - ids_en)
        sys.stderr.write(
            "warning: TODO.md and TODO.pt-BR.md don't list the same tasks "
            f"(only in EN: {only_en or 'none'}; only in PT-BR: {only_pt or 'none'})\n"
        )

    return HTML_TEMPLATE.format(
        count_backlog=len(columns_en["Backlog"]),
        count_progress=len(columns_en["In Progress"]),
        count_review=len(columns_en["Review"]),
        count_done=len(columns_en["Done"]),
        task_count_hint=task_count_hint(columns_en),
        mermaid_en=html_escape(to_mermaid(columns_en).rstrip("\n")),
        mermaid_pt=html_escape(to_mermaid(columns_pt).rstrip("\n")),
        mermaid_version=MERMAID_VERSION,
    )


def main(argv: Optional[list[str]] = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("input", nargs="?", default="TODO.md", type=Path, help="Board file to read (default: TODO.md)")
    parser.add_argument("-o", "--output", type=Path, default=None, help="Write the result here instead of stdout")
    parser.add_argument(
        "--html",
        action="store_true",
        help="Emit the full docs/kanban.html page (reads INPUT plus its pt-BR sibling) instead of a Mermaid block",
    )
    args = parser.parse_args(argv)

    if args.html:
        pt_path = counterpart_path(args.input)
        if args.input.name.endswith(PT_BR_SUFFIX):
            en_path, pt_path = pt_path, args.input
        else:
            en_path = args.input
        if pt_path is None or not en_path.is_file() or not pt_path.is_file():
            sys.stderr.write(
                f"error: --html needs both {en_path} and its pt-BR sibling ({pt_path}) to exist\n"
            )
            return 1

        columns_en = parse_board(en_path.read_text(encoding="utf-8"))
        columns_pt = parse_board(pt_path.read_text(encoding="utf-8"))
        output_path = args.output or (en_path.parent / "docs" / "kanban.html")
        output_path.write_text(build_html(columns_en, columns_pt), encoding="utf-8")
        print(f"Wrote {output_path}")
        return 0

    text = args.input.read_text(encoding="utf-8")
    columns = parse_board(text)
    body = f"```mermaid\n{to_mermaid(columns)}```\n"

    if args.output:
        args.output.write_text(body, encoding="utf-8")
    else:
        sys.stdout.write(body)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
