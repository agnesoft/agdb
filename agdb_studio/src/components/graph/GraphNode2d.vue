<script lang="ts" setup>
import { type PropType, computed } from "vue";
import { type Node } from "@/composables/graph/composable/node";

const props = defineProps({
    node: {
        type: Object as PropType<Node>,
        required: true,
    },
    scale: {
        type: Number,
        required: true,
    },
});

const coordinates = computed(() => {
    return props.node.getCoordinates();
});

const style = computed<StyleObject>(() => {
    const left = coordinates.value.x * props.scale + props.scale;
    const top = coordinates.value.y * props.scale + props.scale;
    return {
        left: `${left}px`,
        top: `${top}px`,
    };
});
</script>

<template>
    <div class="graph-node" :style="style"></div>
</template>

<style lang="less" scoped>
.graph-node {
    position: absolute;
    width: 30px;
    height: 30px;
    background-color: #ffcf82;
    background-image: radial-gradient(#ffcf82, #ff8b07);
    border: 2px solid #ff8b07;
    border-radius: 50%;
    transform: translate(-50%, -50%);
    cursor: pointer;
    transition: filter 0.3s ease-in-out;
}

.graph-node:hover {
    filter: brightness(0.9) saturate(1.2) contrast(1.2);
}
</style>
