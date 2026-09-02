#!/usr/bin/env node
/**
 * Conversor determinístico de Obsidian Canvas (.canvas) para Markdown + Mermaid.
 *
 * Existe para que a parte mecânica do processo descrito em SKILL.md (containment
 * geométrico, geração de IDs, escaping, orientação do diagrama, checklist de
 * completude) não dependa da interpretação de um modelo de linguagem — só a
 * escrita de prosa livre (introdução adicional, diagramas de sequência do
 * Passo 7 opcional) fica a cargo do modelo.
 *
 * Uso:
 *   node convert.ts <entrada.canvas> [saida.md]
 *
 * Requer Node.js 22.6+ (roda .ts nativamente). Se o `node <arquivo>.ts` falhar
 * no seu ambiente, rode com `npx tsx convert.ts ...`.
 *
 * Sem dependências externas — só módulos nativos do Node.
 */

import { readFileSync, writeFileSync } from "node:fs";
import { basename, extname } from "node:path";

// ---------------------------------------------------------------------------
// Tipos do formato .canvas
// ---------------------------------------------------------------------------

type NodeType = "text" | "group" | "file" | "link";

interface CanvasNode {
  id: string;
  type: NodeType;
  x: number;
  y: number;
  width: number;
  height: number;
  text?: string;
  label?: string;
  file?: string;
  url?: string;
  color?: string;
}

interface CanvasEdge {
  id: string;
  fromNode: string;
  fromSide?: string;
  toNode: string;
  toSide?: string;
  label?: string;
  color?: string;
}

interface CanvasFile {
  nodes: CanvasNode[];
  edges: CanvasEdge[];
}

// ---------------------------------------------------------------------------
// Passo 0 — leitura
// ---------------------------------------------------------------------------

function readCanvas(path: string): CanvasFile {
  const raw = readFileSync(path, "utf-8");
  const parsed = JSON.parse(raw) as Partial<CanvasFile>;
  if (!Array.isArray(parsed.nodes) || !Array.isArray(parsed.edges)) {
    throw new Error(
      "Arquivo .canvas inválido: esperava { nodes: [...], edges: [...] }",
    );
  }
  return { nodes: parsed.nodes, edges: parsed.edges };
}

// ---------------------------------------------------------------------------
// Passo 1 — containment geométrico (quem pertence a qual grupo)
// ---------------------------------------------------------------------------

interface Rect {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
}

function rect(n: CanvasNode): Rect {
  return { x1: n.x, y1: n.y, x2: n.x + n.width, y2: n.y + n.height };
}

function contains(outer: Rect, inner: Rect): boolean {
  return (
    inner.x1 >= outer.x1 &&
    inner.x2 <= outer.x2 &&
    inner.y1 >= outer.y1 &&
    inner.y2 <= outer.y2
  );
}

function area(r: Rect): number {
  return (r.x2 - r.x1) * (r.y2 - r.y1);
}

/** Mapa id -> id do grupo pai mais interno (ou undefined se avulso/top-level). */
function buildParentMap(nodes: CanvasNode[]): Map<string, string> {
  const groups = nodes.filter((n) => n.type === "group");
  const parent = new Map<string, string>();

  for (const n of nodes) {
    const nRect = rect(n);
    let best: { id: string; area: number } | undefined;
    for (const g of groups) {
      if (g.id === n.id) continue;
      const gRect = rect(g);
      if (!contains(gRect, nRect)) continue;
      const a = area(gRect);
      if (!best || a < best.area) best = { id: g.id, area: a };
    }
    if (best) parent.set(n.id, best.id);
  }
  return parent;
}

// ---------------------------------------------------------------------------
// Passo 2 — classificar text node: bloco descritivo vs. componente
// ---------------------------------------------------------------------------

function isDescriptiveBlock(n: CanvasNode, hasParent: boolean): boolean {
  if (n.type !== "text" || hasParent) return false;
  const text = n.text ?? "";
  return text.length > 200 || /(^|\n)\s*#/.test(text);
}

// ---------------------------------------------------------------------------
// Passo 4 — slug de ID e rótulo do Mermaid
// ---------------------------------------------------------------------------

const STOPWORDS = new Set([
  "de",
  "da",
  "do",
  "das",
  "dos",
  "e",
  "a",
  "o",
  "as",
  "os",
  "em",
  "para",
  "com",
  "no",
  "na",
  "nos",
  "nas",
  "um",
  "uma",
]);

function stripAccents(s: string): string {
  return s.normalize("NFD").replace(/[̀-ͯ]/g, "");
}

function slugify(rawLabel: string, used: Set<string>): string {
  const words = stripAccents(rawLabel)
    .match(/[A-Za-z0-9]+/g)
    ?.filter((w) => !STOPWORDS.has(w.toLowerCase())) ?? [];

  let base = words.slice(0, 2).join("_").toUpperCase();
  if (!base) base = "NODE";
  if (/^[0-9]/.test(base)) base = `N_${base}`;

  let slug = base;
  let i = 2;
  while (used.has(slug)) {
    slug = `${base}_${i}`;
    i += 1;
  }
  used.add(slug);
  return slug;
}

function nodeSourceText(n: CanvasNode): string {
  if (n.type === "text") return n.text ?? "";
  if (n.type === "file") return `Arquivo: ${n.file ?? ""}`;
  if (n.type === "link") return `Link: ${n.url ?? ""}`;
  return n.label ?? "";
}

/** Primeira frase/linha do texto, sem nenhum escaping — uso em Markdown puro (tabelas). */
function firstSentencePlain(text: string): string {
  const lines = text.split(/\r?\n/).filter((l) => l.trim().length > 0);
  if (lines.length <= 1) {
    const singleLine = lines[0] ?? text;
    const dotIdx = singleLine.indexOf(". ");
    return dotIdx > 0 ? singleLine.slice(0, dotIdx) : singleLine;
  }
  return lines.join(" ");
}

/** Passo 4.2/4.3: mesma extração acima, mas com `<br/>` entre linhas e aspas/ângulos
 *  escapados — só para uso dentro de rótulos Mermaid (que passam por um renderer HTML
 *  internamente; um `<`/`>` cru pode ser interpretado como início de tag). */
function shortLabel(text: string): string {
  const lines = text.split(/\r?\n/).filter((l) => l.trim().length > 0);
  const label = lines.length <= 1 ? firstSentencePlain(text) : lines.join("<br/>");
  return escapeMermaidLabel(label);
}

function escapeMermaidLabel(s: string): string {
  return s.replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "'");
}

/** Passo 4.4: trunca rótulos longos — nunca quebra artificialmente com `<br/>`. */
function truncateLabel(label: string, max = 60): string {
  if (label.includes("<br/>") || label.length <= max) return label;
  return `${label.slice(0, max - 1).trimEnd()}…`;
}

// ---------------------------------------------------------------------------
// Passo 5 — montagem do flowchart
// ---------------------------------------------------------------------------

interface Tree {
  groups: Map<string, CanvasNode>; // id -> group node
  childrenOf: Map<string, string[]>; // group id -> child node ids (any order of insertion)
  topLevel: string[]; // ids (groups or loose diagram nodes) with no parent
}

function buildTree(
  nodes: CanvasNode[],
  parentOf: Map<string, string>,
  diagramNodeIds: Set<string>,
): Tree {
  const groups = new Map<string, CanvasNode>();
  for (const n of nodes) if (n.type === "group") groups.set(n.id, n);

  const childrenOf = new Map<string, string[]>();
  const topLevel: string[] = [];

  const relevant = nodes.filter(
    (n) => n.type === "group" || diagramNodeIds.has(n.id),
  );

  for (const n of relevant) {
    const parent = parentOf.get(n.id);
    if (parent) {
      const list = childrenOf.get(parent) ?? [];
      list.push(n.id);
      childrenOf.set(parent, list);
    } else {
      topLevel.push(n.id);
    }
  }

  return { groups, childrenOf, topLevel };
}

function determineOrientation(nodes: CanvasNode[]): "LR" | "TD" {
  if (nodes.length === 0) return "LR";
  const minX = Math.min(...nodes.map((n) => n.x));
  const maxX = Math.max(...nodes.map((n) => n.x + n.width));
  const minY = Math.min(...nodes.map((n) => n.y));
  const maxY = Math.max(...nodes.map((n) => n.y + n.height));
  return maxX - minX >= maxY - minY ? "LR" : "TD";
}

function renderFlowchart(
  tree: Tree,
  nodesById: Map<string, CanvasNode>,
  slugOf: Map<string, string>,
  edges: CanvasEdge[],
  orientation: "LR" | "TD",
): string {
  const lines: string[] = [`flowchart ${orientation}`];

  const renderNode = (id: string, indent: string) => {
    const n = nodesById.get(id)!;
    const slug = slugOf.get(id)!;
    if (n.type === "group") {
      lines.push(`${indent}subgraph ${slug}["${escapeMermaidLabel(n.label ?? "")}"]`);
      for (const childId of tree.childrenOf.get(id) ?? []) {
        renderNode(childId, `${indent}    `);
      }
      lines.push(`${indent}end`);
    } else {
      const label = truncateLabel(shortLabel(nodeSourceText(n)));
      lines.push(`${indent}${slug}["${label}"]`);
    }
  };

  for (const id of tree.topLevel) renderNode(id, "    ");
  lines.push("");

  for (const e of edges) {
    const fromSlug = slugOf.get(e.fromNode);
    const toSlug = slugOf.get(e.toNode);
    if (!fromSlug || !toSlug) continue; // aresta órfã — já reportada como warning
    if (e.label) {
      lines.push(`    ${fromSlug} -- "${escapeMermaidLabel(e.label)}" --> ${toSlug}`);
    } else {
      lines.push(`    ${fromSlug} --> ${toSlug}`);
    }
  }

  return lines.join("\n");
}

// ---------------------------------------------------------------------------
// Passo 3/6 — tabelas de componentes por grupo
// ---------------------------------------------------------------------------

function renderTables(
  tree: Tree,
  nodesById: Map<string, CanvasNode>,
  minHeading: number,
): string {
  const sections: string[] = [];

  const renderGroup = (groupId: string, depth: number) => {
    const g = nodesById.get(groupId)!;
    const heading = "#".repeat(Math.min(depth + minHeading, 6));
    const rows: string[] = [];
    const subgroups: string[] = [];

    for (const childId of tree.childrenOf.get(groupId) ?? []) {
      const child = nodesById.get(childId)!;
      if (child.type === "group") {
        subgroups.push(childId);
      } else {
        const full = nodeSourceText(child).replace(/\r?\n/g, " ");
        const short = firstSentencePlain(nodeSourceText(child));
        rows.push(`| ${short} | ${full} |`);
      }
    }

    let section = `${heading} ${g.label ?? "(sem nome)"}\n`;
    if (rows.length > 0) {
      section += `\n| Componente | Descrição |\n|---|---|\n${rows.join("\n")}\n`;
    }
    sections.push(section);

    for (const sub of subgroups) renderGroup(sub, depth + 1);
  };

  for (const id of tree.topLevel) {
    if (nodesById.get(id)!.type === "group") renderGroup(id, 0);
  }

  // Nós avulsos (fora de qualquer grupo) que não sejam bloco descritivo.
  const looseRows: string[] = [];
  for (const id of tree.topLevel) {
    const n = nodesById.get(id)!;
    if (n.type === "group") continue;
    const full = nodeSourceText(n).replace(/\r?\n/g, " ");
    const short = firstSentencePlain(nodeSourceText(n));
    looseRows.push(`| ${short} | ${full} |`);
  }
  if (looseRows.length > 0) {
    const heading = "#".repeat(Math.min(minHeading, 6));
    sections.push(
      `${heading} Componentes avulsos\n\n| Componente | Descrição |\n|---|---|\n${looseRows.join("\n")}\n`,
    );
  }

  return sections.join("\n");
}

// ---------------------------------------------------------------------------
// Introdução / título (Passo 6)
// ---------------------------------------------------------------------------

function extractTitleAndBody(descriptiveTexts: string[], canvasPath: string) {
  let title: string | undefined;
  const bodies: string[] = [];

  for (const text of descriptiveTexts) {
    const headingMatch = text.match(/^#\s+(.+)\r?\n\r?\n?([\s\S]*)$/);
    if (headingMatch && !title) {
      title = headingMatch[1].trim();
      bodies.push(headingMatch[2].trimStart());
    } else if (headingMatch) {
      bodies.push(headingMatch[2].trimStart());
    } else {
      bodies.push(text);
    }
  }

  if (!title) title = basename(canvasPath, extname(canvasPath));
  return { title, body: bodies.join("\n\n") };
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

function main() {
  const [, , inputPath, outputPathArg] = process.argv;
  if (!inputPath) {
    console.error("Uso: node convert.ts <entrada.canvas> [saida.md]");
    process.exit(1);
  }
  const outputPath =
    outputPathArg ?? inputPath.replace(/\.canvas$/i, "") + ".md";

  const canvas = readCanvas(inputPath);
  const nodesById = new Map(canvas.nodes.map((n) => [n.id, n] as const));

  const parentOf = buildParentMap(canvas.nodes);

  const descriptiveIds = new Set<string>();
  const diagramNodeIds = new Set<string>();
  for (const n of canvas.nodes) {
    if (n.type === "group") continue;
    if (isDescriptiveBlock(n, parentOf.has(n.id))) {
      descriptiveIds.add(n.id);
    } else {
      diagramNodeIds.add(n.id);
    }
  }

  const tree = buildTree(canvas.nodes, parentOf, diagramNodeIds);

  const slugOf = new Map<string, string>();
  const usedSlugs = new Set<string>();
  for (const n of canvas.nodes) {
    if (n.type === "group") {
      slugOf.set(n.id, slugify(n.label ?? "GRUPO", usedSlugs));
    } else if (diagramNodeIds.has(n.id)) {
      slugOf.set(n.id, slugify(nodeSourceText(n), usedSlugs));
    }
  }

  const diagramAndGroupNodes = canvas.nodes.filter(
    (n) => n.type === "group" || diagramNodeIds.has(n.id),
  );
  const orientation = determineOrientation(diagramAndGroupNodes);

  const flowchart = renderFlowchart(tree, nodesById, slugOf, canvas.edges, orientation);
  const tables = renderTables(tree, nodesById, 2);

  const descriptiveSorted = canvas.nodes
    .filter((n) => descriptiveIds.has(n.id))
    .sort((a, b) => a.y - b.y || a.x - b.x)
    .map((n) => n.text ?? "");

  const { title, body } = extractTitleAndBody(descriptiveSorted, inputPath);

  const parts = [`# ${title}`];
  if (body.trim()) parts.push(body.trim());
  parts.push(`## Visão geral da arquitetura\n\n\`\`\`mermaid\n${flowchart}\n\`\`\``);
  parts.push(tables.trim());

  const markdown = parts.filter(Boolean).join("\n\n") + "\n";
  writeFileSync(outputPath, markdown, "utf-8");

  // ---- Relatório / checklist (Passo 8) — para o modelo revisar, nunca para o .md ----
  const orphanEdges = canvas.edges.filter(
    (e) => !nodesById.has(e.fromNode) || !nodesById.has(e.toNode),
  );
  const groupCount = canvas.nodes.filter((n) => n.type === "group").length;

  console.log(`Escrito em: ${outputPath}`);
  console.log(`Orientação escolhida: ${orientation}`);
  console.log(`Grupos: ${groupCount}`);
  console.log(`Nós de diagrama: ${diagramNodeIds.size}`);
  console.log(`Blocos descritivos: ${descriptiveIds.size}`);
  console.log(`Arestas traduzidas: ${canvas.edges.length - orphanEdges.length}/${canvas.edges.length}`);
  if (orphanEdges.length > 0) {
    console.warn(
      `AVISO: ${orphanEdges.length} aresta(s) referenciam nó inexistente e foram ignoradas: ${orphanEdges
        .map((e) => e.id)
        .join(", ")}`,
    );
  }
  if (descriptiveIds.size > 1) {
    console.warn(
      `AVISO: mais de um bloco descritivo encontrado (${descriptiveIds.size}) — todos foram concatenados em ordem de posição.`,
    );
  }
  for (const n of canvas.nodes) {
    if (n.type === "group" && (tree.childrenOf.get(n.id) ?? []).length === 0) {
      console.warn(`AVISO: grupo "${n.label}" está vazio.`);
    }
  }
  console.log(
    "\nO Passo 7 (diagramas de sequência) NÃO foi gerado por este script — é opcional e requer julgamento; adicione manualmente só se fizer sentido.",
  );
}

main();
