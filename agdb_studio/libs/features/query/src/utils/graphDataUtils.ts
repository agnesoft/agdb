import type {
  DbElement,
  DbKeyValue,
  DbValue,
  QueryResult,
} from "@agnesoft/agdb_api";
import { convertToNativeValue } from "@agnesoft/agdb_api";
import type {
  GraphData,
  GraphElementSource,
  GraphLink,
  GraphNode,
} from "../composables/types";

/** Converts a DbValue union to a display-friendly string. */
export const dbValueToString = (value: DbValue): string => {
  const native = convertToNativeValue(value);
  if (Array.isArray(native)) {
    return native.map(String).join(", ");
  }
  return String(native ?? "");
};

/** Key-values as display strings. */
export const toDisplayValues = (
  values: DbKeyValue[],
): { key: string; value: string }[] =>
  values.map((kv) => ({
    key: dbValueToString(kv.key),
    value: dbValueToString(kv.value),
  }));

/**
 * Transforms `DbElement[]` into the `GraphData` format consumed by 3d-force-graph.
 *
 * - Positive ids are nodes.
 * - Negative ids are edges (`from` → source node, `to` → target node).
 * - Zero ids are skipped (logically invalid in agdb).
 */
export const convertElementsToGraphData = (
  elements: DbElement[],
  source: GraphElementSource,
  resultIndexes?: Map<number, number[]>,
): GraphData => {
  const nodes: GraphNode[] = [];
  const links: GraphLink[] = [];

  for (const el of elements) {
    const values = toDisplayValues(el.values);
    const indexes = resultIndexes?.get(el.id);
    const origin = indexes ? { resultIndexes: indexes } : {};

    if (el.id > 0) {
      nodes.push({
        id: el.id,
        values,
        rawValues: el.values,
        source,
        ...origin,
      });
    } else if (el.id < 0) {
      links.push({
        id: el.id,
        values,
        rawValues: el.values,
        source: el.from,
        target: el.to,
        elementSource: source,
        ...origin,
      });
    }
  }

  return { nodes, links };
};

export type DedupedElement = {
  element: DbElement;
  /** Indexes of the results (queries of the batch) that returned the element. */
  resultIndexes: number[];
};

/**
 * One entry per element id across all results, in order of first
 * appearance. When several queries return the same element, their values
 * are merged by key (later ones win), as each may select different keys.
 */
export const dedupeResultElements = (
  results: QueryResult[],
): DedupedElement[] => {
  const byId = new Map<
    number | string,
    { element: DbElement; resultIndexes: number[] }
  >();
  for (const [resultIndex, result] of results.entries()) {
    for (const [elementIndex, el] of result.elements.entries()) {
      // Id 0 is not an element (e.g. a count or an index list): never merged.
      const key = el.id === 0 ? `0:${resultIndex}:${elementIndex}` : el.id;
      const seen = byId.get(key);
      if (!seen) {
        byId.set(key, { element: el, resultIndexes: [resultIndex] });
        continue;
      }
      if (seen.resultIndexes.at(-1) !== resultIndex) {
        seen.resultIndexes.push(resultIndex);
      }
      const values = new Map(
        seen.element.values.map((kv) => [JSON.stringify(kv.key), kv]),
      );
      for (const kv of el.values) values.set(JSON.stringify(kv.key), kv);
      seen.element = { ...seen.element, values: [...values.values()] };
    }
  }
  return [...byId.values()];
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

/** Drops links whose source or target node is not in the node set. */
export const filterDanglingLinks = (data: GraphData): GraphData => {
  const nodeIds = new Set(data.nodes.map((n) => n.id));
  return {
    nodes: data.nodes,
    links: data.links.filter(
      (link) => nodeIds.has(link.source) && nodeIds.has(link.target),
    ),
  };
};
