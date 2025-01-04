<script lang="ts" setup>
import { computed, provide } from "vue";
import { getRows } from "@/composables/table/tableData";
import AgdbTableRow from "@/components/base/table/AgdbTableRow.vue";
import AgdbTableHeader from "./AgdbTableHeader.vue";
import { getTableColumns } from "@/composables/table/tableConfig";
import { type TRow } from "@/composables/table/types";
import {
    INJECT_KEY_COLUMNS,
    INJECT_KEY_TABLE_NAME,
} from "@/composables/table/constants";

const props = defineProps({
    name: {
        type: [Symbol, String],
        required: true,
    },
});

const rows = computed(() => {
    return getRows(props.name);
});
const columns = computed(() => {
    return getTableColumns<TRow>(props.name);
});
const tableKey = computed(() => props.name);
provide(INJECT_KEY_TABLE_NAME, tableKey);
provide(INJECT_KEY_COLUMNS, columns);
</script>

<template>
    <div class="agdb-table">
        <AgdbTableHeader :tableKey="name" />
        <template v-for="row in rows" :key="row[0]">
            <AgdbTableRow :row="row[1]" :columns="columns" />
        </template>
    </div>
</template>

<style lang="less" scoped>
.agdb-table {
    display: grid;
    padding: 1rem;
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    margin: 0 auto;
    max-width: 100%;
    overflow: auto;
}
.agdb-table ::v-deep(.columns) {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 1rem;
    padding: 0.5rem;
    white-space: nowrap;
    &.expandable {
        grid-template-columns: repeat(auto-fill, minmax(150px, 1fr)) 50px;
    }
}
</style>
