<script lang="ts" setup>
import { computed, type PropType, provide } from "vue";
import { type Column, type TRow } from "@/composables/table/types";
import { INJECT_KEY_ROW } from "@/composables/table/constants";
import AgdbCell from "./AgdbCell.vue";

const props = defineProps({
    row: {
        type: Object as PropType<TRow>,
        required: true,
    },
    columns: {
        type: Object as PropType<Map<string, Column<TRow>>>,
        required: true,
    },
});
const cellKeys = computed(() => {
    return Object.keys(props.row);
});

const rowData = computed(() => {
    return props.row;
});
provide(INJECT_KEY_ROW, rowData);
</script>

<template>
    <div class="agdb-table-row columns">
        <div v-for="cellKey in cellKeys" :key="cellKey">
            <AgdbCell :cell-key="cellKey" />
        </div>
    </div>
</template>

<style lang="less" scoped></style>
