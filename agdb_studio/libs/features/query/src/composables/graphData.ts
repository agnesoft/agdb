import { computed, type Ref } from "vue";
import { useQueryStore } from "./queryStore";
import type { GraphData, GraphElement } from "./types";
import {
  convertElementsToGraphData,
  filterDanglingLinks,
  mergeGraphData,
} from "../utils/graphDataUtils";

/**
 * Reactive graph data of a query: main results merged with context results
 * and on-demand expanded elements (both shown as "context").
 */
export const useGraphData = (queryId: Ref<string | undefined> | undefined) => {
  const queryStore = useQueryStore();

  const query = computed(() => {
    if (!queryId?.value) return undefined;
    return queryStore.getQuery(queryId.value)?.value;
  });

  /** Merged graph data including links whose endpoints are not loaded. */
  const unfilteredGraphData = computed<GraphData>(() => {
    const results = query.value?.results;
    if (!results) return { nodes: [], links: [] };

    const activeTab = query.value?.lastExecutedTab ?? "exec";
    const mainElements = (results[activeTab] ?? []).flatMap((r) => r.elements);
    const queryGraphData = convertElementsToGraphData(mainElements, "query");

    const contextElements = [
      ...(results.context ?? []),
      ...(results.expanded ?? []),
    ].flatMap((r) => r.elements);

    if (contextElements.length === 0) {
      return queryGraphData;
    }

    return mergeGraphData(
      queryGraphData,
      convertElementsToGraphData(contextElements, "context"),
    );
  });

  const graphData = computed<GraphData>(() =>
    filterDanglingLinks(unfilteredGraphData.value),
  );

  /** Lookup of every known element by id (including dangling links). */
  const elementsById = computed(() => {
    const map = new Map<number, GraphElement>();
    for (const node of unfilteredGraphData.value.nodes) map.set(node.id, node);
    for (const link of unfilteredGraphData.value.links) map.set(link.id, link);
    return map;
  });

  return { query, graphData, elementsById };
};
