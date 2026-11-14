import type { GraphElement, GraphLink, GraphNode } from "../composables/types";
import { toDisplayValues } from "./graphDataUtils";

/** Colors used by the graph, resolved from CSS custom properties. */
export type GraphTheme = {
  background: string;
  node: string;
  contextNode: string;
  edge: string;
  edgeAccent: string;
  contextEdge: string;
  hover: string;
  selected: string;
};

export const GRAPH_THEME_FALLBACK: GraphTheme = {
  background: "#f8f8f8",
  node: "#ffc835",
  contextNode: "#b5b5b5",
  edge: "#8a8a8a",
  edgeAccent: "#dd8300",
  contextEdge: "#c8c8c8",
  hover: "#ffa02c",
  selected: "#007bff",
};

const THEME_VARS = [
  ["background", "--graph-background"],
  ["node", "--graph-node-color"],
  ["contextNode", "--graph-context-node-color"],
  ["edge", "--graph-edge-color"],
  ["edgeAccent", "--graph-edge-color-accent"],
  ["contextEdge", "--graph-context-edge-color"],
  ["hover", "--graph-hover-color"],
  ["selected", "--graph-selected-color"],
] as const satisfies readonly (readonly [keyof GraphTheme, string])[];

/** Reads the graph theme from CSS custom properties on `el`. */
export const readGraphTheme = (el: Element): GraphTheme => {
  const style = getComputedStyle(el);
  const theme = { ...GRAPH_THEME_FALLBACK };
  for (const [key, cssVar] of THEME_VARS) {
    const value = style.getPropertyValue(cssVar).trim();
    if (value) theme[key] = value;
  }
  return theme;
};

/** Interaction state that influences element styling. */
export type HighlightState = {
  hoveredNodeId?: number;
  hoveredLinkId?: number;
  selectedId?: number;
  pendingSourceId?: number;
  /** Nodes highlighted by the current hover (hovered node or edge endpoints). */
  highlightNodes: Set<number>;
  /** Links highlighted by the current hover (hovered edge or node's edges). */
  highlightLinks: Set<number>;
};

export const emptyHighlightState = (): HighlightState => ({
  highlightNodes: new Set(),
  highlightLinks: new Set(),
});

/** 3d-force-graph replaces link endpoints with node objects at runtime. */
export const linkEndpointId = (endpoint: unknown): number => {
  if (endpoint && typeof endpoint === "object" && "id" in endpoint) {
    return Number(endpoint.id);
  }
  return Number(endpoint ?? 0);
};

/** Computes highlighted nodes/links for a hovered node or link. */
export const computeHighlight = (
  hovered: { node?: GraphNode; link?: GraphLink },
  links: GraphLink[],
): Pick<HighlightState, "highlightNodes" | "highlightLinks"> => {
  const highlightNodes = new Set<number>();
  const highlightLinks = new Set<number>();
  if (hovered.node) {
    const id = hovered.node.id;
    highlightNodes.add(id);
    for (const link of links) {
      const source = linkEndpointId(link.source);
      const target = linkEndpointId(link.target);
      if (source === id || target === id) {
        highlightLinks.add(link.id);
        highlightNodes.add(source);
        highlightNodes.add(target);
      }
    }
  } else if (hovered.link) {
    highlightLinks.add(hovered.link.id);
    highlightNodes.add(linkEndpointId(hovered.link.source));
    highlightNodes.add(linkEndpointId(hovered.link.target));
  }
  return { highlightNodes, highlightLinks };
};

const hasHover = (state: HighlightState): boolean =>
  state.hoveredNodeId !== undefined || state.hoveredLinkId !== undefined;

/** Applies an alpha channel to a hex or rgb() color. Other formats are returned unchanged. */
export const withAlpha = (color: string, alpha: number): string => {
  const hex = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(color.trim());
  if (hex?.[1]) {
    const digits =
      hex[1].length === 3
        ? hex[1]
            .split("")
            .map((d) => d + d)
            .join("")
        : hex[1];
    const r = parseInt(digits.slice(0, 2), 16);
    const g = parseInt(digits.slice(2, 4), 16);
    const b = parseInt(digits.slice(4, 6), 16);
    return `rgba(${r}, ${g}, ${b}, ${alpha})`;
  }
  const rgb = /^rgba?\(\s*(\d+)[,\s]+(\d+)[,\s]+(\d+)/i.exec(color.trim());
  if (rgb) {
    return `rgba(${rgb[1]}, ${rgb[2]}, ${rgb[3]}, ${alpha})`;
  }
  return color;
};

const DIM_ALPHA = 0.25;

export const getNodeColor = (
  node: GraphNode,
  state: HighlightState,
  theme: GraphTheme,
): string => {
  if (node.id === state.selectedId || node.id === state.pendingSourceId) {
    return theme.selected;
  }
  if (node.id === state.hoveredNodeId) {
    return theme.hover;
  }
  const base = node.source === "context" ? theme.contextNode : theme.node;
  if (hasHover(state) && !state.highlightNodes.has(node.id)) {
    return withAlpha(base, DIM_ALPHA);
  }
  return base;
};

export const getLinkColor = (
  link: GraphLink,
  state: HighlightState,
  theme: GraphTheme,
): string => {
  if (link.id === state.selectedId) {
    return theme.selected;
  }
  if (state.highlightLinks.has(link.id)) {
    return theme.edgeAccent;
  }
  const base =
    link.elementSource === "context" ? theme.contextEdge : theme.edge;
  if (hasHover(state)) {
    return withAlpha(base, DIM_ALPHA);
  }
  return base;
};

export const LINK_WIDTH = 1.5;
export const LINK_WIDTH_HIGHLIGHT = 3;

export const getLinkWidth = (link: GraphLink, state: HighlightState): number =>
  link.id === state.selectedId || state.highlightLinks.has(link.id)
    ? LINK_WIDTH_HIGHLIGHT
    : LINK_WIDTH;

/**
 * Assigns a curvature to every link so that parallel edges between the same
 * pair of nodes (and self-loops) do not overlap. Single edges stay straight.
 */
export const computeLinkCurvatures = (
  links: GraphLink[],
): Map<number, number> => {
  const groups = new Map<string, GraphLink[]>();
  for (const link of links) {
    const a = linkEndpointId(link.source);
    const b = linkEndpointId(link.target);
    const key = a < b ? `${a}:${b}` : `${b}:${a}`;
    const group = groups.get(key) ?? [];
    group.push(link);
    groups.set(key, group);
  }

  const curvatures = new Map<number, number>();
  for (const group of groups.values()) {
    group.forEach((link, index) => {
      const selfLoop =
        linkEndpointId(link.source) === linkEndpointId(link.target);
      if (selfLoop) {
        curvatures.set(link.id, 0.5 + index * 0.2);
      } else if (group.length === 1) {
        curvatures.set(link.id, 0);
      } else {
        // Spread parallel edges symmetrically; opposite directions get the
        // same sign because the curve is relative to each link's direction.
        const step = 0.25 * (Math.floor(index / 2) + 1);
        const forward =
          linkEndpointId(link.source) < linkEndpointId(link.target);
        const sign = index % 2 === 0 ? 1 : -1;
        curvatures.set(link.id, (forward ? sign : -sign) * step);
      }
    });
  }
  return curvatures;
};

const escapeHtml = (text: string): string =>
  text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");

export const TOOLTIP_MAX_VALUES = 5;

type TooltipValue = { key: string; value: string; fromQuery?: boolean };

/** Stored values first, then values only the query returned (marked). */
const tooltipEntries = (element: GraphElement): TooltipValue[] => [
  ...element.values,
  ...toDisplayValues(element.queryOnlyValues ?? []).map((v) => ({
    ...v,
    fromQuery: true,
  })),
];

const tooltipValues = (element: GraphElement): string => {
  const values = tooltipEntries(element);
  const shown = values
    .slice(0, TOOLTIP_MAX_VALUES)
    .map(
      (v) =>
        `<div class="graph-tooltip-row"><span class="graph-tooltip-key">${escapeHtml(v.key)}</span> ${escapeHtml(v.value)}${v.fromQuery ? ' <span class="graph-tooltip-tag">query</span>' : ""}</div>`,
    )
    .join("");
  const more =
    values.length > TOOLTIP_MAX_VALUES
      ? `<div class="graph-tooltip-more">+${values.length - TOOLTIP_MAX_VALUES} more</div>`
      : "";
  const loading =
    element.valuesState === "loading"
      ? '<div class="graph-tooltip-more">Loading stored values…</div>'
      : "";
  return shown + more + loading;
};

/** HTML tooltip for a node. All user data is escaped. */
export const nodeTooltip = (node: GraphNode): string =>
  `<div class="graph-tooltip"><div class="graph-tooltip-title">Node ${node.id}</div>${tooltipValues(node)}</div>`;

/** HTML tooltip for a link. All user data is escaped. */
export const linkTooltip = (link: GraphLink): string =>
  `<div class="graph-tooltip"><div class="graph-tooltip-title">Edge ${link.id}</div><div class="graph-tooltip-row">${linkEndpointId(link.source)} → ${linkEndpointId(link.target)}</div>${tooltipValues(link)}</div>`;
