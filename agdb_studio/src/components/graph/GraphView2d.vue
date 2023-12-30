<script lang="ts" setup>
import { type PropType, computed, ref } from "vue";
import GraphEdge2d from "@/components/graph/GraphEdge2d.vue";
import GraphNode2d from "@/components/graph/GraphNode2d.vue";
import useForceDirectedGraph from "@/composables/graph/composable/forceDirectedGraph";

const props = defineProps({
    graphData: {
        type: Object as PropType<Graph>,
        required: true,
    },
});

const graph = useForceDirectedGraph({ is2d: true });
graph.loadGraph(props.graphData);
graph.simulate();

const graphNodes = computed(() => {
    return graph.getNodes();
});

const graphEdges = computed(() => {
    return graph.getEdges();
});

const scale = ref(400);

const style = computed<StyleObject>(() => {
    return {
        "--size": `${scale.value * 2}px`,
    };
});
</script>

<template>
    <div class="graph-view">
        <div class="graph-wrapper" :style="style">
            <div class="graph-edges">
                <GraphEdge2d
                    v-for="edge in graphEdges"
                    :key="edge.getId()"
                    :edge="edge"
                    :scale="scale"
                />
            </div>
            <div class="graph-nodes">
                <GraphNode2d
                    v-for="node in graphNodes"
                    :key="node.getId()"
                    :node="node"
                    :scale="scale"
                />
            </div>
        </div>
    </div>
</template>

<style lang="less" scoped>
.graph-view {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
}
.graph-wrapper {
    position: relative;
    overflow: auto;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 100px;
    height: 100px;
    max-width: 100dvw;
    max-height: 80dvh;
    width: var(--size);
    height: var(--size);
}
.graph-edges,
.graph-nodes {
    position: absolute;
    top: 0;
    left: 0;
    width: var(--size);
    height: var(--size);
}
</style>
