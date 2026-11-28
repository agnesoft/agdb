import type { GraphElement, GraphLink, GraphNode } from "../composables/types";
import { toDisplayValues } from "./graphDataUtils";

/** The gradient scales a gradient or distance rule can use. */
export const RAMP_NAMES = ["teal", "purple", "rose", "multi"] as const;
export type RampName = (typeof RAMP_NAMES)[number];
/** The scale of rules saved before they could pick one. */
export const DEFAULT_RAMP: RampName = "teal";

export const isRampName = (value: unknown): value is RampName =>
  RAMP_NAMES.some((name) => name === value);

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
  /** Nodes no colour applies to, while colours are in use. */
  uncolored: string;
  /** Categorical colours for colour rules, in fixed order. */
  palette: string[];
  /** Scales for gradients, each from low to high. */
  ramps: Record<RampName, string[]>;
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
  uncolored: "#b5b5b5",
  palette: [
    "#17890b",
    "#2dbad8",
    "#eda100",
    "#9752fe",
    "#f92240",
    "#7d376e",
    "#3f38c6",
  ],
  ramps: {
    teal: [
      "#04c5b5",
      "#0cab9d",
      "#039286",
      "#087970",
      "#03625a",
      "#044b45",
      "#013631",
    ],
    purple: [
      "#bb91ff",
      "#a47ae7",
      "#8e63ce",
      "#794db6",
      "#64379e",
      "#501f86",
      "#3d006f",
    ],
    rose: [
      "#fb78b0",
      "#e2629a",
      "#ca4b85",
      "#b23471",
      "#9a1a5d",
      "#7f0049",
      "#600136",
    ],
    multi: [
      "#acb900",
      "#4caf47",
      "#089474",
      "#087679",
      "#025a73",
      "#013d73",
      "#230177",
    ],
  },
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
  ["uncolored", "--graph-uncolored-color"],
] as const satisfies readonly (readonly [keyof GraphTheme, string])[];

/** Reads the graph theme from CSS custom properties on `el`. */
export const readGraphTheme = (el: Element): GraphTheme => {
  const style = getComputedStyle(el);
  const theme = {
    ...GRAPH_THEME_FALLBACK,
    palette: [...GRAPH_THEME_FALLBACK.palette],
  };
  for (const [key, cssVar] of THEME_VARS) {
    const value = style.getPropertyValue(cssVar).trim();
    if (value) theme[key] = value;
  }
  theme.palette = theme.palette.map(
    (fallback, i) =>
      style.getPropertyValue(`--graph-palette-${i + 1}`).trim() || fallback,
  );
  const ramp = (name: RampName) =>
    GRAPH_THEME_FALLBACK.ramps[name].map(
      (fallback, i) =>
        style.getPropertyValue(`--graph-ramp-${name}-${i + 1}`).trim() ||
        fallback,
    );
  theme.ramps = {
    teal: ramp("teal"),
    purple: ramp("purple"),
    rose: ramp("rose"),
    multi: ramp("multi"),
  };
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

/** The red, green and blue of a hex or rgb() color, if it is one. */
const parseRgb = (color: string): [number, number, number] | undefined => {
  const hex = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(color.trim());
  if (hex?.[1]) {
    const digits =
      hex[1].length === 3
        ? hex[1]
            .split("")
            .map((d) => d + d)
            .join("")
        : hex[1];
    const at = (i: number) => parseInt(digits.slice(i, i + 2), 16);
    return [at(0), at(2), at(4)];
  }
  const rgb = /^rgba?\(\s*(\d+)[,\s]+(\d+)[,\s]+(\d+)/i.exec(color.trim());
  return rgb ? [Number(rgb[1]), Number(rgb[2]), Number(rgb[3])] : undefined;
};

/** Applies an alpha channel to a hex or rgb() color. Other formats are returned unchanged. */
export const withAlpha = (color: string, alpha: number): string => {
  const rgb = parseRgb(color);
  return rgb ? `rgba(${rgb[0]}, ${rgb[1]}, ${rgb[2]}, ${alpha})` : color;
};

/** The gradient scale as a CSS background, low to high along `direction`. */
export const rampGradient = (
  ramp: readonly string[],
  direction = "to right",
): string => `linear-gradient(${direction}, ${ramp.join(", ")})`;

/**
 * The colour at `t` (0 low, 1 high) along a scale, blended between its two
 * nearest steps. Steps that are not hex or rgb() are not blended.
 */
export const rampColor = (ramp: readonly string[], t: number): string => {
  const position = Math.min(1, Math.max(0, t)) * (ramp.length - 1);
  const index = Math.floor(position);
  const low = ramp[index] ?? "";
  const high = ramp[index + 1];
  const a = parseRgb(low);
  const b = high === undefined ? undefined : parseRgb(high);
  if (!a || !b) return low;
  const mix = position - index;
  return `#${a
    .map((v, i) =>
      Math.round(v + (b[i]! - v) * mix)
        .toString(16)
        .padStart(2, "0"),
    )
    .join("")}`;
};

const DIM_ALPHA = 0.25;

/**
 * `custom` replaces the default colour, e.g. from a colour rule; it is kept
 * on hover, as the hover colour could be mistaken for a palette colour.
 * While colours are in use (`colored`), nodes without one are neutral.
 */
export const getNodeColor = (
  node: GraphNode,
  state: HighlightState,
  theme: GraphTheme,
  custom?: string,
  colored = false,
): string => {
  if (node.id === state.selectedId || node.id === state.pendingSourceId) {
    return theme.selected;
  }
  if (node.id === state.hoveredNodeId) {
    return custom ?? theme.hover;
  }
  const base =
    custom ??
    (colored
      ? theme.uncolored
      : node.source === "context"
        ? theme.contextNode
        : theme.node);
  if (hasHover(state) && !state.highlightNodes.has(node.id)) {
    return withAlpha(base, DIM_ALPHA);
  }
  return base;
};

export const getLinkColor = (
  link: GraphLink,
  state: HighlightState,
  theme: GraphTheme,
  custom?: string,
): string => {
  if (link.id === state.selectedId) {
    return theme.selected;
  }
  if (state.highlightLinks.has(link.id)) {
    return custom ?? theme.edgeAccent;
  }
  const base =
    custom ??
    (link.elementSource === "context" ? theme.contextEdge : theme.edge);
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

const tooltipTitle = (kind: string, element: GraphElement): string =>
  `<div class="graph-tooltip-title">${kind} ${element.id}${
    element.alias
      ? ` <span class="graph-tooltip-alias">${escapeHtml(element.alias)}</span>`
      : ""
  }</div>`;

const TOOLTIP_HINT = '<div class="graph-tooltip-hint">Shift+click to pin</div>';

/** HTML tooltip for a node. All user data is escaped. */
export const nodeTooltip = (node: GraphNode): string =>
  `<div class="graph-tooltip">${tooltipTitle("Node", node)}${tooltipValues(node)}${TOOLTIP_HINT}</div>`;

/** HTML tooltip for a link. All user data is escaped. */
export const linkTooltip = (link: GraphLink): string =>
  `<div class="graph-tooltip">${tooltipTitle("Edge", link)}<div class="graph-tooltip-row">${linkEndpointId(link.source)} → ${linkEndpointId(link.target)}</div>${tooltipValues(link)}${TOOLTIP_HINT}</div>`;
