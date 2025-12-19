<script lang="ts" setup>
import ForceGraph3D, { type ForceGraph3DInstance } from "3d-force-graph";
import { onMounted, ref } from "vue";
import { graphDataMock } from "../../mock/graphData";

type GraphNode = {
  id: string;
  values: { key: string; value: string }[];
};

const graphContainer = ref<HTMLElement | null>(null);

const graphData = ref(graphDataMock);

let _graphInstance: ForceGraph3DInstance;

onMounted(() => {
  /* v8 ignore else -- @preserve */
  if (graphContainer.value) {
    _graphInstance = new ForceGraph3D(graphContainer.value)
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
        (link) =>
          `Source: ${(link.source as GraphNode).id}\nTarget: ${(link.target as GraphNode).id}`,
      )
      .height(graphContainer.value.clientHeight)
      .width(graphContainer.value.clientWidth);
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
