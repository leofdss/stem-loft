#!/usr/bin/env python3
"""Conversor determinístico de Obsidian Canvas (.canvas) para Markdown + Mermaid.

Existe para que a parte mecânica do processo descrito em SKILL.md (containment
geométrico, geração de IDs, escaping, orientação do diagrama, checklist de
completude) não dependa da interpretação de um modelo de linguagem — só a
escrita de prosa livre (introdução adicional, diagramas de sequência do
Passo 7 opcional) fica a cargo do modelo.

Uso:
    python3 convert.py <entrada.canvas> [saida.md]

Sem dependências externas — só biblioteca padrão do Python 3.
"""
from __future__ import annotations

import json
import re
import sys
import unicodedata
from dataclasses import dataclass, field
from pathlib import Path

# ---------------------------------------------------------------------------
# Tipos do formato .canvas
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
# Passo 0 — leitura
# ---------------------------------------------------------------------------


def read_canvas(path: str) -> CanvasFile:
    raw = Path(path).read_text(encoding="utf-8")
    parsed = json.loads(raw)
    if not isinstance(parsed.get("nodes"), list) or not isinstance(parsed.get("edges"), list):
        raise ValueError("Arquivo .canvas inválido: esperava { nodes: [...], edges: [...] }")

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
# Passo 1 — containment geométrico (quem pertence a qual grupo)
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
    """Mapa id -> id do grupo pai mais interno (ausente se avulso/top-level)."""
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
# Passo 2 — classificar text node: bloco descritivo vs. componente
# ---------------------------------------------------------------------------


def is_descriptive_block(n: CanvasNode, has_parent: bool) -> bool:
    if n.type != "text" or has_parent:
        return False
    text = n.text or ""
    return len(text) > 200 or re.search(r"(^|\n)\s*#", text) is not None


# ---------------------------------------------------------------------------
# Passo 4 — slug de ID e rótulo do Mermaid
# ---------------------------------------------------------------------------

STOPWORDS = {
    "de", "da", "do", "das", "dos", "e", "a", "o", "as", "os",
    "em", "para", "com", "no", "na", "nos", "nas", "um", "uma",
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
        return f"Arquivo: {n.file or ''}"
    if n.type == "link":
        return f"Link: {n.url or ''}"
    return n.label or ""


def first_sentence_plain(text: str) -> str:
    """Primeira frase/linha do texto, sem nenhum escaping — uso em Markdown puro (tabelas)."""
    lines = [l for l in re.split(r"\r?\n", text) if l.strip()]
    if len(lines) <= 1:
        single_line = lines[0] if lines else text
        dot_idx = single_line.find(". ")
        return single_line[:dot_idx] if dot_idx > 0 else single_line
    return " ".join(lines)


def short_label(text: str) -> str:
    """Passo 4.2/4.3: mesma extração acima, mas com `<br/>` entre linhas e aspas/ângulos
    escapados — só para uso dentro de rótulos Mermaid (que passam por um renderer HTML
    internamente; um `<`/`>` cru pode ser interpretado como início de tag)."""
    lines = [l for l in re.split(r"\r?\n", text) if l.strip()]
    label = first_sentence_plain(text) if len(lines) <= 1 else "<br/>".join(lines)
    return escape_mermaid_label(label)


def escape_mermaid_label(s: str) -> str:
    return s.replace("<", "&lt;").replace(">", "&gt;").replace('"', "'")


def truncate_label(label: str, max_len: int = 60) -> str:
    """Passo 4.4: trunca rótulos longos — nunca quebra artificialmente com `<br/>`."""
    if "<br/>" in label or len(label) <= max_len:
        return label
    return label[: max_len - 1].rstrip() + "…"


# ---------------------------------------------------------------------------
# Passo 5 — montagem do flowchart
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
            continue  # aresta órfã — já reportada como warning
        if e.label:
            lines.append(f'    {from_slug} -- "{escape_mermaid_label(e.label)}" --> {to_slug}')
        else:
            lines.append(f"    {from_slug} --> {to_slug}")

    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Passo 3/6 — tabelas de componentes por grupo
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

        section = f"{heading} {g.label or '(sem nome)'}\n"
        if rows:
            section += "\n| Componente | Descrição |\n|---|---|\n" + "\n".join(rows) + "\n"
        sections.append(section)

        for sub in subgroups:
            render_group(sub, depth + 1)

    for node_id in tree.top_level:
        if nodes_by_id[node_id].type == "group":
            render_group(node_id, 0)

    # Nós avulsos (fora de qualquer grupo) que não sejam bloco descritivo.
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
            f"{heading} Componentes avulsos\n\n| Componente | Descrição |\n|---|---|\n"
            + "\n".join(loose_rows)
            + "\n"
        )

    return "\n".join(sections)


# ---------------------------------------------------------------------------
# Introdução / título (Passo 6)
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
        print("Uso: python3 convert.py <entrada.canvas> [saida.md]", file=sys.stderr)
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
            slug_of[n.id] = slugify(n.label or "GRUPO", used_slugs)
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
    parts.append(f"## Visão geral da arquitetura\n\n```mermaid\n{flowchart}\n```")
    parts.append(tables.strip())

    markdown = "\n\n".join(p for p in parts if p) + "\n"
    Path(output_path).write_text(markdown, encoding="utf-8")

    # ---- Relatório / checklist (Passo 8) — para o modelo revisar, nunca para o .md ----
    orphan_edges = [e for e in canvas.edges if e.fromNode not in nodes_by_id or e.toNode not in nodes_by_id]
    group_count = sum(1 for n in canvas.nodes if n.type == "group")

    print(f"Escrito em: {output_path}")
    print(f"Orientação escolhida: {orientation}")
    print(f"Grupos: {group_count}")
    print(f"Nós de diagrama: {len(diagram_node_ids)}")
    print(f"Blocos descritivos: {len(descriptive_ids)}")
    print(f"Arestas traduzidas: {len(canvas.edges) - len(orphan_edges)}/{len(canvas.edges)}")
    if orphan_edges:
        ids = ", ".join(e.id for e in orphan_edges)
        print(
            f"AVISO: {len(orphan_edges)} aresta(s) referenciam nó inexistente e foram ignoradas: {ids}",
            file=sys.stderr,
        )
    if len(descriptive_ids) > 1:
        print(
            f"AVISO: mais de um bloco descritivo encontrado ({len(descriptive_ids)}) — "
            "todos foram concatenados em ordem de posição.",
            file=sys.stderr,
        )
    for n in canvas.nodes:
        if n.type == "group" and not tree.children_of.get(n.id):
            print(f'AVISO: grupo "{n.label}" está vazio.', file=sys.stderr)
    print(
        "\nO Passo 7 (diagramas de sequência) NÃO foi gerado por este script — é opcional "
        "e requer julgamento; adicione manualmente só se fizer sentido."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
