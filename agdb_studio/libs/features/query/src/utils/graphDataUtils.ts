import type { Components } from "@agnesoft/agdb_api";
import { convertToNativeValue } from "@agnesoft/agdb_api";
import type {
  GraphData,
  GraphElementSource,
  GraphLink,
  GraphNode,
} from "../composables/types";

/** Converts a DbValue union to a display-friendly string. */
export const dbValueToString = (value: Components.Schemas.DbValue): string => {
  const native = convertToNativeValue(value);
  if (Array.isArray(native)) {
    return native.map(String).join(", ");
  }
  return String(native ?? "");
};

/**
 * Transforms `DbElement[]` into the `GraphData` format consumed by 3d-force-graph.
 *
 * - Positive ids are nodes.
 * - Negative ids are edges (`from` → source node, `to` → target node).
 * - Zero ids are skipped (logically invalid in agdb).
 */
export const convertElementsToGraphData = (
  elements: Components.Schemas.DbElement[],
  source: GraphElementSource,
): GraphData => {
  const nodes: GraphNode[] = [];
  const links: GraphLink[] = [];

  for (const el of elements) {
    const values = el.values.map((kv) => ({
      key: dbValueToString(kv.key),
      value: dbValueToString(kv.value),
    }));

    if (el.id > 0) {
      nodes.push({ id: el.id, values, source });
    } else if (el.id < 0) {
      links.push({
        id: el.id,
        values,
        source: el.from,
        target: el.to,
        elementSource: source,
      });
    }
  }

  return { nodes, links };
};

/**
 * Merges query and context graph data.
 * Query elements take priority — if the same id appears in both,
 * the query version (non-greyed) is kept.
 */
export const mergeGraphData = (
  queryData: GraphData,
  contextData: GraphData,
): GraphData => {
  const nodeMap = new Map<number, GraphNode>();
  const linkMap = new Map<number, GraphLink>();

  for (const node of queryData.nodes) {
    nodeMap.set(node.id, node);
  }
  for (const link of queryData.links) {
    linkMap.set(link.id, link);
  }

  for (const node of contextData.nodes) {
    if (!nodeMap.has(node.id)) {
      nodeMap.set(node.id, node);
    }
  }
  for (const link of contextData.links) {
    if (!linkMap.has(link.id)) {
      linkMap.set(link.id, link);
    }
  }

  return {
    nodes: Array.from(nodeMap.values()),
    links: Array.from(linkMap.values()),
  };
};
