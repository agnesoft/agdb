import { describe, expect, it, vi } from "vitest";
vi.mock("@agnesoft/agdb_api", async (importOriginal) => {
  return await importOriginal<typeof import("@agnesoft/agdb_api")>();
});
import type { GraphLink, GraphNode } from "../composables/types";
import {
  GRAPH_THEME_FALLBACK,
  LINK_WIDTH,
  LINK_WIDTH_HIGHLIGHT,
  TOOLTIP_MAX_VALUES,
  computeHighlight,
  computeLinkCurvatures,
  emptyHighlightState,
  getLinkColor,
  getLinkWidth,
  getNodeColor,
  linkEndpointId,
  linkTooltip,
  nodeTooltip,
  readGraphTheme,
  withAlpha,
} from "./graphStyle";

const theme = GRAPH_THEME_FALLBACK;

const node = (
  id: number,
  source: GraphNode["source"] = "query",
): GraphNode => ({
  id,
  values: [],
  rawValues: [],
  source,
});

const link = (
  id: number,
  source: number,
  target: number,
  elementSource: GraphLink["elementSource"] = "query",
): GraphLink => ({
  id,
  values: [],
  rawValues: [],
  source,
  target,
  elementSource,
});

describe("graphStyle", () => {
  describe("readGraphTheme", () => {
    it("reads css variables and falls back for missing ones", () => {
      const el = document.createElement("div");
      el.style.setProperty("--graph-edge-color", " #123456 ");
      document.body.appendChild(el);
      const result = readGraphTheme(el);
      expect(result.edge).toBe("#123456");
      expect(result.node).toBe(GRAPH_THEME_FALLBACK.node);
      el.remove();
    });
  });

  describe("linkEndpointId", () => {
    it("handles numbers, node objects and missing values", () => {
      expect(linkEndpointId(3)).toBe(3);
      expect(linkEndpointId({ id: 4 })).toBe(4);
      expect(linkEndpointId(undefined)).toBe(0);
    });
  });

  describe("computeHighlight", () => {
    const links = [link(-1, 1, 2), link(-2, 2, 3), link(-3, 3, 4)];

    it("highlights a hovered node with its edges and neighbours", () => {
      const result = computeHighlight({ node: node(2) }, links);
      expect([...result.highlightLinks]).toEqual([-1, -2]);
      expect([...result.highlightNodes].sort()).toEqual([1, 2, 3]);
    });

    it("highlights a hovered link and its endpoints", () => {
      const result = computeHighlight({ link: link(-3, 3, 4) }, links);
      expect([...result.highlightLinks]).toEqual([-3]);
      expect([...result.highlightNodes]).toEqual([3, 4]);
    });

    it("returns empty sets without hover", () => {
      const result = computeHighlight({}, links);
      expect(result.highlightLinks.size).toBe(0);
      expect(result.highlightNodes.size).toBe(0);
    });
  });

  describe("withAlpha", () => {
    it("converts hex and rgb colors", () => {
      expect(withAlpha("#fff", 0.5)).toBe("rgba(255, 255, 255, 0.5)");
      expect(withAlpha("#102030", 0.1)).toBe("rgba(16, 32, 48, 0.1)");
      expect(withAlpha("rgb(1, 2, 3)", 0.2)).toBe("rgba(1, 2, 3, 0.2)");
      expect(withAlpha("rgba(1 2 3 / 1)", 0.2)).toBe("rgba(1, 2, 3, 0.2)");
    });

    it("returns unknown formats unchanged", () => {
      expect(withAlpha("red", 0.5)).toBe("red");
    });
  });

  describe("getNodeColor", () => {
    it("uses base colors without interaction", () => {
      const state = emptyHighlightState();
      expect(getNodeColor(node(1), state, theme)).toBe(theme.node);
      expect(getNodeColor(node(1, "context"), state, theme)).toBe(
        theme.contextNode,
      );
    });

    it("prioritizes selection and pending source over hover", () => {
      const state = {
        ...emptyHighlightState(),
        selectedId: 1,
        pendingSourceId: 2,
        hoveredNodeId: 1,
      };
      expect(getNodeColor(node(1), state, theme)).toBe(theme.selected);
      expect(getNodeColor(node(2), state, theme)).toBe(theme.selected);
    });

    it("colors the hovered node and dims unrelated nodes", () => {
      const state = {
        ...emptyHighlightState(),
        hoveredNodeId: 1,
        highlightNodes: new Set([1, 2]),
      };
      expect(getNodeColor(node(1), state, theme)).toBe(theme.hover);
      expect(getNodeColor(node(2), state, theme)).toBe(theme.node);
      expect(getNodeColor(node(3), state, theme)).toBe(
        withAlpha(theme.node, 0.25),
      );
    });
  });

  describe("getLinkColor and getLinkWidth", () => {
    it("uses base colors and widths without interaction", () => {
      const state = emptyHighlightState();
      expect(getLinkColor(link(-1, 1, 2), state, theme)).toBe(theme.edge);
      expect(getLinkColor(link(-1, 1, 2, "context"), state, theme)).toBe(
        theme.contextEdge,
      );
      expect(getLinkWidth(link(-1, 1, 2), state)).toBe(LINK_WIDTH);
    });

    it("highlights selected and hovered links, dims others", () => {
      const state = {
        ...emptyHighlightState(),
        selectedId: -1,
        hoveredLinkId: -2,
        highlightLinks: new Set([-2]),
      };
      expect(getLinkColor(link(-1, 1, 2), state, theme)).toBe(theme.selected);
      expect(getLinkColor(link(-2, 1, 2), state, theme)).toBe(theme.edgeAccent);
      expect(getLinkColor(link(-3, 1, 2), state, theme)).toBe(
        withAlpha(theme.edge, 0.25),
      );
      expect(getLinkWidth(link(-1, 1, 2), state)).toBe(LINK_WIDTH_HIGHLIGHT);
      expect(getLinkWidth(link(-2, 1, 2), state)).toBe(LINK_WIDTH_HIGHLIGHT);
      expect(getLinkWidth(link(-3, 1, 2), state)).toBe(LINK_WIDTH);
    });
  });

  describe("computeLinkCurvatures", () => {
    it("keeps single edges straight and curves parallel edges and loops", () => {
      const curvatures = computeLinkCurvatures([
        link(-1, 1, 2),
        link(-2, 3, 4),
        link(-3, 4, 3),
        link(-4, 5, 5),
        link(-5, 3, 4),
      ]);
      expect(curvatures.get(-1)).toBe(0);
      expect(curvatures.get(-2)).toBe(0.25);
      // Reverse direction: same sign flip gives the opposite geometric side.
      expect(curvatures.get(-3)).toBe(0.25);
      expect(curvatures.get(-5)).toBe(0.5);
      expect(curvatures.get(-4)).toBe(0.5);
    });
  });

  describe("tooltips", () => {
    it("renders escaped node values and truncates long lists", () => {
      const values = Array.from({ length: TOOLTIP_MAX_VALUES + 2 }, (_, i) => ({
        key: `k${i}`,
        value: "<b>x</b>",
      }));
      const html = nodeTooltip({ ...node(7), values });
      expect(html).toContain("Node 7");
      expect(html).toContain("&lt;b&gt;x&lt;/b&gt;");
      expect(html).not.toContain("<b>x</b>");
      expect(html).toContain("+2 more");
    });

    it("renders link endpoints", () => {
      const html = linkTooltip({
        ...link(-3, 1, 2),
        values: [{ key: "a&b", value: "'\"" }],
      });
      expect(html).toContain("Edge -3");
      expect(html).toContain("1 → 2");
      expect(html).toContain("a&amp;b");
      expect(html).toContain("&#39;&quot;");
      expect(html).not.toContain("more");
    });

    it("lists stored values, then marked values only the query returned", () => {
      const html = nodeTooltip({
        ...node(7),
        values: [{ key: "name", value: "Alice" }],
        valuesState: "loaded",
        queryOnlyValues: [
          { key: { String: "<edge_count>" }, value: { U64: 3 } },
        ],
      });
      expect(html.indexOf("name")).toBeLessThan(html.indexOf("edge_count"));
      expect(html).toContain("&lt;edge_count&gt;");
      expect(html.match(/graph-tooltip-tag/g)).toHaveLength(1);
      expect(html).not.toContain("Loading");
    });

    it("counts query-only values when truncating", () => {
      const html = linkTooltip({
        ...link(-3, 1, 2),
        values: Array.from({ length: TOOLTIP_MAX_VALUES }, (_, i) => ({
          key: `k${i}`,
          value: "v",
        })),
        queryOnlyValues: [{ key: { String: "extra" }, value: { I64: 1 } }],
      });
      expect(html).toContain("+1 more");
      expect(html).not.toContain("extra");
    });

    it("says when stored values are still loading", () => {
      expect(nodeTooltip({ ...node(7), valuesState: "loading" })).toContain(
        "Loading stored values",
      );
    });
  });
});
