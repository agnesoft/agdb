<script lang="ts" setup>
import { defineProps, computed, type PropType } from "vue";
import { type Column, type TRow } from "@/composables/table/types";

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
const getFromattedValue = (key: string) => {
    const column = props.columns.get(key);
    const value = props.row[key];
    if (column?.valueFormatter) {
        return column.valueFormatter(value);
    }
    return value;
};
</script>

<template>
    <div class="agdb-table-row columns">
        <div v-for="cellKey in cellKeys" :key="cellKey">
            <p>{{ getFromattedValue(cellKey) }}</p>
        </div>
    </div>
</template>

<style lang="less" scoped></style>
