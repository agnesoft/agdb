<script lang="ts" setup>
import ForceGraph3D, { type ForceGraph3DInstance } from "3d-force-graph";
import { onMounted, onUnmounted, ref } from "vue";
import { graphDataMock } from "../../mock/graphData";

type GraphNode = {
  id: number;
  values: { key: string; value: string }[];
};

const graphContainer = ref<HTMLElement | null>(null);

const graphData = ref(graphDataMock);

let graphInstance: ForceGraph3DInstance;

onMounted(() => {
  /* v8 ignore else -- @preserve */
  if (graphContainer.value) {
    graphInstance = new ForceGraph3D(graphContainer.value)
      .graphData(graphData.value)
      .nodeAutoColorBy("id")
      .linkAutoColorBy("source")
      .nodeLabel(
        /* v8 ignore next -- @preserve */
        (node) =>
          `ID: ${node.id}\nName: ${(node as GraphNode).values
            .map((v) => `${v.key}: ${v.value}`)
            .join("\n")}`,
      )
      .linkLabel(
        /* v8 ignore next -- @preserve */
        (link) => {
          const sourceId =
            link.source && typeof link.source === "object"
              ? (link.source as GraphNode).id
              : String(link.source ?? "");
          const targetId =
            link.target && typeof link.target === "object"
              ? (link.target as GraphNode).id
              : String(link.target ?? "");
          return `Source: ${sourceId}\nTarget: ${targetId}`;
        },
      )
      .height(graphContainer.value.clientHeight)
      .width(graphContainer.value.clientWidth);
  }
});

onUnmounted(() => {
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
