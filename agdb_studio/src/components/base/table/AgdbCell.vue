<script lang="ts" setup>
import {
    INJECT_KEY_COLUMNS,
    INJECT_KEY_ROW,
} from "@/composables/table/constants";
import type { Column, TRow } from "@/composables/table/types";
import { computed, inject } from "vue";

const props = defineProps({
    cellKey: {
        type: String,
        required: true,
    },
});
const columns = inject<Map<string, Column<TRow>>>(INJECT_KEY_COLUMNS);
const row = inject<TRow>(INJECT_KEY_ROW);

const column = computed(() => columns?.get(props.cellKey));
const value = computed(() => row?.[props.cellKey]);

const formattedValue = computed(() => {
    if (!column.value || !value.value) {
        return "";
    }
    if (column.value?.valueFormatter) {
        return column.value.valueFormatter(value.value);
    }
    return value.value;
});
</script>

<template>
    <div>
        <div v-if="column?.cellComponent">
            <component :is="column.cellComponent" :value="value" />
        </div>
        <div v-else-if="column?.actions">
            <AgdbCellMenu :actions="column.actions" />
        </div>
        <div v-else>
            <p>{{ formattedValue }}</p>
        </div>
    </div>
</template>

<style lang="less" scoped></style>
