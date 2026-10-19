#!/usr/bin/env python3
"""Generate the task board -- docs/kanban.html -- from OpenSpec.

Inputs (and nothing else, so the output is a pure function of the repo):
  - openspec/changes/<id>/proposal.md and proposal.pt-BR.md -- the title
    (`# TASK-NNN — Title`) in each language;
  - openspec/board.toml -- each active change's status/branch;
  - openspec/changes/archive/<date>-<id>/ -- archived changes are Done.

Usage:
    python3 scripts/openspec_to_kanban.py               # (re)write docs/kanban.html
    python3 scripts/openspec_to_kanban.py --check       # exit 1 if docs/kanban.html is stale
    python3 scripts/openspec_to_kanban.py --mermaid     # print the EN Mermaid block
    python3 scripts/openspec_to_kanban.py --mermaid --lang pt-BR

Never hand-edit docs/kanban.html; change the inputs and re-run this.
"""

from __future__ import annotations

import argparse
import re
import sys
import tomllib
from pathlib import Path
from typing import Optional

TITLE_RE = re.compile(r"^#\s+(TASK-\d+)\s+—\s+(.+?)\s*$", re.MULTILINE)
ARCHIVE_PREFIX_RE = re.compile(r"^\d{4}-\d{2}-\d{2}-")

# Fixes the column order regardless of the order changes are found in.
KNOWN_COLUMNS = ("Backlog", "In Progress", "Review", "Done")
STATUS_TO_COLUMN = {"backlog": "Backlog", "in-progress": "In Progress", "review": "Review"}

ROOT = Path(__file__).resolve().parent.parent
OPENSPEC = ROOT / "openspec"
OUTPUT = ROOT / "docs" / "kanban.html"

# Pinned exact version, per project policy of not trusting a floating "latest".
# 10.9.x and earlier do NOT ship the `kanban` diagram type at all (verified by
# grepping the minified bundle) -- if you bump this, verify the new version's
# cdnjs bundle still contains the string "kanban" before shipping it, since a
# missing diagram type fails silently at render time with a cryptic Mermaid
# parse error, not a load error.
MERMAID_VERSION = "11.15.0"

Task = dict[str, Optional[str]]
Columns = dict[str, list[Task]]


def read_title(change_dir: Path, lang: str) -> Optional[tuple[str, str]]:
    name = "proposal.md" if lang == "en" else f"proposal.{lang}.md"
    path = change_dir / name
    if not path.is_file():
        return None
    match = TITLE_RE.search(path.read_text(encoding="utf-8"))
    return (match.group(1), match.group(2)) if match else None


def load_board() -> dict:
    path = OPENSPEC / "board.toml"
    return tomllib.loads(path.read_text(encoding="utf-8")) if path.is_file() else {}


def task_number(task: Task) -> int:
    match = re.search(r"(\d+)$", str(task["id"]))
    return int(match.group(1)) if match else 0


def parse_openspec(lang: str) -> Columns:
    columns: Columns = {name: [] for name in KNOWN_COLUMNS}
    board = load_board()
    changes = OPENSPEC / "changes"
    for change_dir in sorted(p for p in changes.iterdir() if p.is_dir() and p.name != "archive"):
        title = read_title(change_dir, lang)
        if title is None:
            continue
        meta = board.get(change_dir.name, {})
        column = STATUS_TO_COLUMN.get(meta.get("status", "backlog"), "Backlog")
        columns[column].append({"id": title[0], "title": title[1], "branch": meta.get("branch")})
    archive = changes / "archive"
    for change_dir in sorted(p for p in archive.iterdir() if p.is_dir()) if archive.is_dir() else []:
        title = read_title(change_dir, lang)
        if title is not None:
            columns["Done"].append({"id": title[0], "title": title[1], "branch": None})
    for tasks in columns.values():
        tasks.sort(key=task_number)
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
      <p class="eyebrow">OpenSpec &rarr; Mermaid kanban</p>
      <h1>StemLoft implementation board</h1>
      <p class="lede">
        Generated by <code>scripts/openspec_to_kanban.py</code> from the
        OpenSpec changes under <code>openspec/changes/</code> and
        <code>openspec/board.toml</code> -- this file is a pure function of
        those, never hand-edited. Open it straight from disk, no server
        needed.
      </p>
    </div>
    <div class="lang-toggle" role="group" aria-label="Board language">
      <button type="button" id="btn-en" aria-pressed="true">EN &middot; proposal.md</button>
      <button type="button" id="btn-pt" aria-pressed="false">PT-BR &middot; proposal.pt-BR.md</button>
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
      <span class="file">Source: <strong id="source-file">proposal.md</strong></span>
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
      <b>Keep this in sync</b> &mdash; whenever a change is added, archived,
      renamed, or moves column in <code>openspec/board.toml</code>, run
      <code>python3 scripts/openspec_to_kanban.py</code>. CI and
      <code>scripts/check_changed.py</code> fail when this page is stale.
    </div>
    <div class="col">
      <b>Regenerate</b>
      <div class="cmd">python3 scripts/openspec_to_kanban.py</div>
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
      sourceFile.textContent = 'proposal.md';
    }}
    function showPt() {{
      wrapEn.hidden = true;
      wrapPt.hidden = false;
      btnEn.setAttribute('aria-pressed', 'false');
      btnPt.setAttribute('aria-pressed', 'true');
      sourceFile.textContent = 'proposal.pt-BR.md';
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
            "warning: the EN and pt-BR proposals don't list the same tasks "
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
    parser.add_argument("--check", action="store_true", help="exit 1 if docs/kanban.html is out of date")
    parser.add_argument("--mermaid", action="store_true", help="print the Mermaid block instead of writing the page")
    parser.add_argument("--lang", default="en", choices=("en", "pt-BR"), help="language for --mermaid")
    args = parser.parse_args(argv)

    if args.mermaid:
        sys.stdout.write(f"```mermaid\n{to_mermaid(parse_openspec(args.lang))}```\n")
        return 0

    html = build_html(parse_openspec("en"), parse_openspec("pt-BR"))
    if args.check:
        current = OUTPUT.read_text(encoding="utf-8") if OUTPUT.is_file() else ""
        if current != html:
            print("docs/kanban.html is stale -- run `python3 scripts/openspec_to_kanban.py`.")
            return 1
        return 0
    OUTPUT.write_text(html, encoding="utf-8")
    print(f"Wrote {OUTPUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
