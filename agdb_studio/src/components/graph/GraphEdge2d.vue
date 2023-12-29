<script lang="ts" setup>
import { defineProps, type PropType, computed } from "vue";
import { type Edge } from "@/composables/graph/composable/edge";

const props = defineProps({
    edge: {
        type: Object as PropType<Edge>,
        required: true,
    },
    scale: {
        type: Number,
        required: true,
    },
});

const fromCoordinates = computed(() => {
    return props.edge.getFrom()?.getCoordinates();
});

const toCoordinates = computed(() => {
    return props.edge.getTo()?.getCoordinates();
});

const style = computed<StyleObject>(() => {
    if (!fromCoordinates.value || !toCoordinates.value) {
        return {} as StyleObject;
    }
    // Calculate the line coordinates
    const x1 = fromCoordinates.value.x * props.scale + props.scale;
    const y1 = fromCoordinates.value.y * props.scale + props.scale;
    const x2 = toCoordinates.value.x * props.scale + props.scale;
    const y2 = toCoordinates.value.y * props.scale + props.scale;

    // Calculate the line length
    const length = Math.sqrt(Math.pow(x2 - x1, 2) + Math.pow(y2 - y1, 2));

    // Calculate the line angle
    const angle = Math.atan2(y2 - y1, x2 - x1) * (180 / Math.PI);

    // Return the line style
    return {
        left: `${x1}px`,
        top: `${y1}px`,
        width: `${length}px`,
        transform: `rotate(${angle}deg)`,
    };
});
</script>

<template>
    <div class="graph-edge" :style="style"></div>
</template>

<style lang="less" scoped>
.graph-edge {
    position: absolute;
    height: 0px;
    transform-origin: top left;
    border: 1px solid grey;
    height: 0px;
}
</style>
