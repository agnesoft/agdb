<script lang="ts" setup>
import ForceGraph3D, { type ForceGraph3DInstance } from "3d-force-graph";
import {
  computed,
  inject,
  onMounted,
  onUnmounted,
  ref,
  watch,
  type Ref,
} from "vue";
import { useQueryStore } from "../../composables/queryStore";
import type { GraphNode, GraphLink, GraphData } from "../../composables/types";
import {
  convertElementsToGraphData,
  filterDanglingLinks,
  mergeGraphData,
} from "../../utils/graphDataUtils";

/**
 * 3d-force-graph uses generic node/link objects internally.
 * These helpers cast to our typed variants to avoid repeated `as unknown as` chains.
 */
/* v8 ignore next 2 -- @preserve */
const asGraphNode = (node: object): GraphNode => node as GraphNode;
const asGraphLink = (link: object): GraphLink => link as unknown as GraphLink;

const graphContainer = ref<HTMLElement | null>(null);

const queryId = inject<Ref<string>>("queryId");
const queryStore = useQueryStore();

const query = computed(() => {
  /* v8 ignore next */
  if (!queryId?.value) return undefined;
  return queryStore.getQuery(queryId.value)?.value;
});

const graphData = computed<GraphData>(() => {
  const results = query.value?.results;
  /* v8 ignore next */
  if (!results) return { nodes: [], links: [] };

  const activeTab = query.value?.lastExecutedTab ?? "exec";
  const mainResults = results[activeTab] ?? [];
  const mainElements = mainResults.flatMap((r) => r.elements);
  const queryGraphData = convertElementsToGraphData(mainElements, "query");

  const contextResults = results.context ?? [];
  const contextElements = contextResults.flatMap((r) => r.elements);

  if (contextElements.length === 0) {
    return filterDanglingLinks(queryGraphData);
  }

  const contextGraphData = convertElementsToGraphData(
    contextElements,
    "context",
  );
  return filterDanglingLinks(mergeGraphData(queryGraphData, contextGraphData));
});

// TODO: Colors are resolved once at mount. If the user switches themes after
// mount the graph colors will be stale. A future enhancement could watch for
// theme changes via a MutationObserver on the root element's class list.
const getComputedColor = (varName: string, fallback: string): string => {
  /* v8 ignore next */
  if (!graphContainer.value) return fallback;
  return (
    getComputedStyle(graphContainer.value).getPropertyValue(varName).trim() ||
    fallback
  );
};

let graphInstance: ForceGraph3DInstance;
let zoomTimer: ReturnType<typeof setTimeout> | undefined;

onMounted(() => {
  /* v8 ignore else -- @preserve */
  if (graphContainer.value) {
    const queryNodeColor = getComputedColor("--graph-node-color", "#ffc835");
    const queryEdgeColor = getComputedColor("--graph-edge-color", "#555555");
    const contextNodeColor = getComputedColor(
      "--graph-context-node-color",
      "#888888",
    );
    const contextEdgeColor = getComputedColor(
      "--graph-context-edge-color",
      "#555555",
    );

    graphInstance = new ForceGraph3D(graphContainer.value)
      .graphData(graphData.value)
      .nodeColor(
        /* v8 ignore next -- @preserve */
        (node) =>
          asGraphNode(node).source === "context"
            ? contextNodeColor
            : queryNodeColor,
      )
      .linkColor(
        /* v8 ignore next -- @preserve */
        (link) =>
          asGraphLink(link).elementSource === "context"
            ? contextEdgeColor
            : queryEdgeColor,
      )
      .nodeLabel(
        /* v8 ignore next -- @preserve */
        (node) => {
          const n = asGraphNode(node);
          return `ID: ${n.id}\n${n.values
            .map((v) => `${v.key}: ${v.value}`)
            .join("\n")}`;
        },
      )
      .linkLabel(
        /* v8 ignore next -- @preserve */
        (link) => {
          const sourceId =
            link.source && typeof link.source === "object"
              ? asGraphNode(link.source).id
              : String(link.source ?? "");
          const targetId =
            link.target && typeof link.target === "object"
              ? asGraphNode(link.target).id
              : String(link.target ?? "");
          return `Source: ${sourceId}\nTarget: ${targetId}`;
        },
      )
      .height(graphContainer.value.clientHeight)
      .width(graphContainer.value.clientWidth);
  }
});

watch(graphData, (newData) => {
  /* v8 ignore next -- @preserve */
  if (graphInstance) {
    clearTimeout(zoomTimer);
    graphInstance.graphData(newData);
    if (newData.nodes.length > 0) {
      graphInstance.d3ReheatSimulation();
      // Wait for the simulation to settle a bit before zooming to fit
      zoomTimer = setTimeout(() => graphInstance.zoomToFit(400), 300);
    }
  }
});

onUnmounted(() => {
  clearTimeout(zoomTimer);
  /* v8 ignore else -- @preserve */
  if (graphInstance) {
    graphInstance._destructor();
  }
});
</script>

<template>
  <div ref="graphContainer" class="query-graph"></div>
</template>

<style lang="less" scoped>
.query-graph {
  width: 100%;
  height: 100%;
  max-height: calc(100vh - 14rem);
  min-height: 20rem;
  overflow: hidden;
  border: 1px solid var(--color-border);
  border-radius: 4px;
}
</style>
