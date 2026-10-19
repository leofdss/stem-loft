#!/usr/bin/env python3
"""Deterministic converter from Obsidian Canvas (.canvas) to Markdown + Mermaid.

Exists so that the mechanical part of the process described in SKILL.md
(geometric containment, ID generation, escaping, diagram orientation,
completeness checklist) doesn't depend on a language model's interpretation
— only free-form prose (an extra introduction, the optional Step 7 sequence
diagrams) is left to the model.

Usage:
    python3 convert.py <input.canvas> [output.md]

No external dependencies — only the Python 3 standard library.
"""
from __future__ import annotations

import json
import re
import sys
import unicodedata
from dataclasses import dataclass, field
from pathlib import Path

# ---------------------------------------------------------------------------
# .canvas format types
# ---------------------------------------------------------------------------


@dataclass
class CanvasNode:
    id: str
    type: str
    x: float
    y: float
    width: float
    height: float
    text: str | None = None
    label: str | None = None
    file: str | None = None
    url: str | None = None
    color: str | None = None


@dataclass
class CanvasEdge:
    id: str
    fromNode: str
    toNode: str
    fromSide: str | None = None
    toSide: str | None = None
    label: str | None = None
    color: str | None = None


@dataclass
class CanvasFile:
    nodes: list[CanvasNode] = field(default_factory=list)
    edges: list[CanvasEdge] = field(default_factory=list)


# ---------------------------------------------------------------------------
# Step 0 — reading
# ---------------------------------------------------------------------------


def read_canvas(path: str) -> CanvasFile:
    raw = Path(path).read_text(encoding="utf-8")
    parsed = json.loads(raw)
    if not isinstance(parsed.get("nodes"), list) or not isinstance(parsed.get("edges"), list):
        raise ValueError("Invalid .canvas file: expected { nodes: [...], edges: [...] }")

    nodes = [
        CanvasNode(
            id=n["id"],
            type=n["type"],
            x=n["x"],
            y=n["y"],
            width=n["width"],
            height=n["height"],
            text=n.get("text"),
            label=n.get("label"),
            file=n.get("file"),
            url=n.get("url"),
            color=n.get("color"),
        )
        for n in parsed["nodes"]
    ]
    edges = [
        CanvasEdge(
            id=e["id"],
            fromNode=e["fromNode"],
            toNode=e["toNode"],
            fromSide=e.get("fromSide"),
            toSide=e.get("toSide"),
            label=e.get("label"),
            color=e.get("color"),
        )
        for e in parsed["edges"]
    ]
    return CanvasFile(nodes=nodes, edges=edges)


# ---------------------------------------------------------------------------
# Step 1 — geometric containment (who belongs to which group)
# ---------------------------------------------------------------------------


@dataclass
class Rect:
    x1: float
    y1: float
    x2: float
    y2: float


def rect(n: CanvasNode) -> Rect:
    return Rect(n.x, n.y, n.x + n.width, n.y + n.height)


def contains(outer: Rect, inner: Rect) -> bool:
    return (
        inner.x1 >= outer.x1
        and inner.x2 <= outer.x2
        and inner.y1 >= outer.y1
        and inner.y2 <= outer.y2
    )


def area(r: Rect) -> float:
    return (r.x2 - r.x1) * (r.y2 - r.y1)


def build_parent_map(nodes: list[CanvasNode]) -> dict[str, str]:
    """Map of id -> innermost parent group id (absent if loose/top-level)."""
    groups = [n for n in nodes if n.type == "group"]
    parent: dict[str, str] = {}

    for n in nodes:
        n_rect = rect(n)
        best: tuple[str, float] | None = None
        for g in groups:
            if g.id == n.id:
                continue
            g_rect = rect(g)
            if not contains(g_rect, n_rect):
                continue
            a = area(g_rect)
            if best is None or a < best[1]:
                best = (g.id, a)
        if best:
            parent[n.id] = best[0]
    return parent


# ---------------------------------------------------------------------------
# Step 2 — classify text node: descriptive block vs. component
# ---------------------------------------------------------------------------


def is_descriptive_block(n: CanvasNode, has_parent: bool) -> bool:
    if n.type != "text" or has_parent:
        return False
    text = n.text or ""
    return len(text) > 200 or re.search(r"(^|\n)\s*#", text) is not None


# ---------------------------------------------------------------------------
# Step 4 — Mermaid ID slug and label
# ---------------------------------------------------------------------------

STOPWORDS = {
    "of", "the", "a", "an", "and", "or", "in", "on", "for", "with",
    "to", "from", "at", "by", "as",
}


def strip_accents(s: str) -> str:
    decomposed = unicodedata.normalize("NFD", s)
    return "".join(c for c in decomposed if not unicodedata.combining(c))


def slugify(raw_label: str, used: set[str]) -> str:
    words = [
        w for w in re.findall(r"[A-Za-z0-9]+", strip_accents(raw_label))
        if w.lower() not in STOPWORDS
    ]

    base = "_".join(words[:2]).upper()
    if not base:
        base = "NODE"
    if re.match(r"^[0-9]", base):
        base = f"N_{base}"

    slug = base
    i = 2
    while slug in used:
        slug = f"{base}_{i}"
        i += 1
    used.add(slug)
    return slug


def node_source_text(n: CanvasNode) -> str:
    if n.type == "text":
        return n.text or ""
    if n.type == "file":
        return f"File: {n.file or ''}"
    if n.type == "link":
        return f"Link: {n.url or ''}"
    return n.label or ""


def first_sentence_plain(text: str) -> str:
    """First sentence/line of the text, with no escaping — for use in plain Markdown (tables)."""
    lines = [l for l in re.split(r"\r?\n", text) if l.strip()]
    if len(lines) <= 1:
        single_line = lines[0] if lines else text
        dot_idx = single_line.find(". ")
        return single_line[:dot_idx] if dot_idx > 0 else single_line
    return " ".join(lines)


def short_label(text: str) -> str:
    """Step 4.2/4.3: same extraction as above, but with `<br/>` between lines and
    quotes/angle brackets escaped — only for use inside Mermaid labels (which pass
    through an internal HTML renderer; a raw `<`/`>` could be read as the start of a
    tag)."""
    lines = [l for l in re.split(r"\r?\n", text) if l.strip()]
    if len(lines) <= 1:
        return escape_mermaid_label(first_sentence_plain(text))
    # Escape each line individually so the joining `<br/>` itself isn't escaped.
    return "<br/>".join(escape_mermaid_label(l) for l in lines)


def escape_mermaid_label(s: str) -> str:
    return s.replace("<", "&lt;").replace(">", "&gt;").replace('"', "'")


def truncate_label(label: str, max_len: int = 60) -> str:
    """Step 4.4: truncates long labels — never breaks artificially with `<br/>`."""
    if "<br/>" in label or len(label) <= max_len:
        return label
    return label[: max_len - 1].rstrip() + "…"


# ---------------------------------------------------------------------------
# Step 5 — assembling the flowchart
# ---------------------------------------------------------------------------


@dataclass
class Tree:
    groups: dict[str, CanvasNode]
    children_of: dict[str, list[str]]
    top_level: list[str]


def build_tree(
    nodes: list[CanvasNode],
    parent_of: dict[str, str],
    diagram_node_ids: set[str],
) -> Tree:
    groups = {n.id: n for n in nodes if n.type == "group"}
    children_of: dict[str, list[str]] = {}
    top_level: list[str] = []

    relevant = [n for n in nodes if n.type == "group" or n.id in diagram_node_ids]

    for n in relevant:
        parent = parent_of.get(n.id)
        if parent:
            children_of.setdefault(parent, []).append(n.id)
        else:
            top_level.append(n.id)

    return Tree(groups=groups, children_of=children_of, top_level=top_level)


def determine_orientation(nodes: list[CanvasNode]) -> str:
    if not nodes:
        return "LR"
    min_x = min(n.x for n in nodes)
    max_x = max(n.x + n.width for n in nodes)
    min_y = min(n.y for n in nodes)
    max_y = max(n.y + n.height for n in nodes)
    return "LR" if (max_x - min_x) >= (max_y - min_y) else "TD"


def render_flowchart(
    tree: Tree,
    nodes_by_id: dict[str, CanvasNode],
    slug_of: dict[str, str],
    edges: list[CanvasEdge],
    orientation: str,
) -> str:
    lines: list[str] = [f"flowchart {orientation}"]

    def render_node(node_id: str, indent: str) -> None:
        n = nodes_by_id[node_id]
        slug = slug_of[node_id]
        if n.type == "group":
            lines.append(f'{indent}subgraph {slug}["{escape_mermaid_label(n.label or "")}"]')
            for child_id in tree.children_of.get(node_id, []):
                render_node(child_id, indent + "    ")
            lines.append(f"{indent}end")
        else:
            label = truncate_label(short_label(node_source_text(n)))
            lines.append(f'{indent}{slug}["{label}"]')

    for node_id in tree.top_level:
        render_node(node_id, "    ")
    lines.append("")

    for e in edges:
        from_slug = slug_of.get(e.fromNode)
        to_slug = slug_of.get(e.toNode)
        if not from_slug or not to_slug:
            continue  # orphaned edge — already reported as a warning
        if e.label:
            lines.append(f'    {from_slug} -- "{escape_mermaid_label(e.label)}" --> {to_slug}')
        else:
            lines.append(f"    {from_slug} --> {to_slug}")

    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Steps 3/6 — component tables per group
# ---------------------------------------------------------------------------


def render_tables(tree: Tree, nodes_by_id: dict[str, CanvasNode], min_heading: int) -> str:
    sections: list[str] = []

    def render_group(group_id: str, depth: int) -> None:
        g = nodes_by_id[group_id]
        heading = "#" * min(depth + min_heading, 6)
        rows: list[str] = []
        subgroups: list[str] = []

        for child_id in tree.children_of.get(group_id, []):
            child = nodes_by_id[child_id]
            if child.type == "group":
                subgroups.append(child_id)
            else:
                full = re.sub(r"\r?\n", " ", node_source_text(child))
                short = first_sentence_plain(node_source_text(child))
                rows.append(f"| {short} | {full} |")

        section = f"{heading} {g.label or '(unnamed)'}\n"
        if rows:
            section += "\n| Component | Description |\n|---|---|\n" + "\n".join(rows) + "\n"
        sections.append(section)

        for sub in subgroups:
            render_group(sub, depth + 1)

    for node_id in tree.top_level:
        if nodes_by_id[node_id].type == "group":
            render_group(node_id, 0)

    # Loose nodes (outside any group) that aren't a descriptive block.
    loose_rows: list[str] = []
    for node_id in tree.top_level:
        n = nodes_by_id[node_id]
        if n.type == "group":
            continue
        full = re.sub(r"\r?\n", " ", node_source_text(n))
        short = first_sentence_plain(node_source_text(n))
        loose_rows.append(f"| {short} | {full} |")
    if loose_rows:
        heading = "#" * min(min_heading, 6)
        sections.append(
            f"{heading} Loose components\n\n| Component | Description |\n|---|---|\n"
            + "\n".join(loose_rows)
            + "\n"
        )

    return "\n".join(sections)


# ---------------------------------------------------------------------------
# Introduction / title (Step 6)
# ---------------------------------------------------------------------------


def extract_title_and_body(descriptive_texts: list[str], canvas_path: str) -> tuple[str, str]:
    title: str | None = None
    bodies: list[str] = []

    for text in descriptive_texts:
        heading_match = re.match(r"^#\s+(.+)\r?\n\r?\n?([\s\S]*)$", text)
        if heading_match and title is None:
            title = heading_match.group(1).strip()
            bodies.append(heading_match.group(2).lstrip())
        elif heading_match:
            bodies.append(heading_match.group(2).lstrip())
        else:
            bodies.append(text)

    if title is None:
        title = Path(canvas_path).stem
    return title, "\n\n".join(bodies)


# ---------------------------------------------------------------------------
# main
# ---------------------------------------------------------------------------


def main() -> int:
    argv = sys.argv[1:]
    if not argv:
        print("Usage: python3 convert.py <input.canvas> [output.md]", file=sys.stderr)
        return 1

    input_path = argv[0]
    output_path = argv[1] if len(argv) > 1 else re.sub(r"\.canvas$", "", input_path, flags=re.IGNORECASE) + ".md"

    canvas = read_canvas(input_path)
    nodes_by_id = {n.id: n for n in canvas.nodes}

    parent_of = build_parent_map(canvas.nodes)

    descriptive_ids: set[str] = set()
    diagram_node_ids: set[str] = set()
    for n in canvas.nodes:
        if n.type == "group":
            continue
        if is_descriptive_block(n, n.id in parent_of):
            descriptive_ids.add(n.id)
        else:
            diagram_node_ids.add(n.id)

    tree = build_tree(canvas.nodes, parent_of, diagram_node_ids)

    slug_of: dict[str, str] = {}
    used_slugs: set[str] = set()
    for n in canvas.nodes:
        if n.type == "group":
            slug_of[n.id] = slugify(n.label or "GROUP", used_slugs)
        elif n.id in diagram_node_ids:
            slug_of[n.id] = slugify(node_source_text(n), used_slugs)

    diagram_and_group_nodes = [n for n in canvas.nodes if n.type == "group" or n.id in diagram_node_ids]
    orientation = determine_orientation(diagram_and_group_nodes)

    flowchart = render_flowchart(tree, nodes_by_id, slug_of, canvas.edges, orientation)
    tables = render_tables(tree, nodes_by_id, 2)

    descriptive_sorted = sorted(
        (n for n in canvas.nodes if n.id in descriptive_ids),
        key=lambda n: (n.y, n.x),
    )
    descriptive_texts = [n.text or "" for n in descriptive_sorted]

    title, body = extract_title_and_body(descriptive_texts, input_path)

    parts = [f"# {title}"]
    if body.strip():
        parts.append(body.strip())
    parts.append(f"## Architecture overview\n\n```mermaid\n{flowchart}\n```")
    parts.append(tables.strip())

    markdown = "\n\n".join(p for p in parts if p) + "\n"
    Path(output_path).write_text(markdown, encoding="utf-8")

    # ---- Report / checklist (Step 8) — for the model to review, never for the .md ----
    orphan_edges = [e for e in canvas.edges if e.fromNode not in nodes_by_id or e.toNode not in nodes_by_id]
    group_count = sum(1 for n in canvas.nodes if n.type == "group")

    print(f"Written to: {output_path}")
    print(f"Orientation chosen: {orientation}")
    print(f"Groups: {group_count}")
    print(f"Diagram nodes: {len(diagram_node_ids)}")
    print(f"Descriptive blocks: {len(descriptive_ids)}")
    print(f"Edges translated: {len(canvas.edges) - len(orphan_edges)}/{len(canvas.edges)}")
    if orphan_edges:
        ids = ", ".join(e.id for e in orphan_edges)
        print(
            f"WARNING: {len(orphan_edges)} edge(s) reference a nonexistent node and were skipped: {ids}",
            file=sys.stderr,
        )
    if len(descriptive_ids) > 1:
        print(
            f"WARNING: more than one descriptive block found ({len(descriptive_ids)}) — "
            "all of them were concatenated in position order.",
            file=sys.stderr,
        )
    for n in canvas.nodes:
        if n.type == "group" and not tree.children_of.get(n.id):
            print(f'WARNING: group "{n.label}" is empty.', file=sys.stderr)
    print(
        "\nStep 7 (sequence diagrams) was NOT generated by this script — it's optional "
        "and requires judgment; add it manually only if it makes sense."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
