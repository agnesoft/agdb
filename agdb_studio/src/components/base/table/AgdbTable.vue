<script lang="ts" setup>
import { computed, defineProps, provide } from "vue";
import { getRows } from "@/composables/table/tableData";
import TableRow from "@/components/base/table/AgdbTableRow.vue";
import TableHeader from "./AgdbTableHeader.vue";
import { getTableColumns } from "@/composables/table/tableConfig";
import { type TRow } from "@/composables/table/types";
import {
    INJECT_KEY_COLUMNS,
    INJECT_KEY_TABLE_NAME,
} from "@/composables/table/constants";

const props = defineProps({
    name: {
        type: String,
        required: true,
    },
});

const rows = computed(() => {
    return getRows(props.name);
});
const columns = computed(() => {
    return getTableColumns<TRow>(props.name);
});

provide(INJECT_KEY_TABLE_NAME, props.name);
provide(INJECT_KEY_COLUMNS, columns);
</script>

<template>
    <div class="agdb-table">
        <TableHeader :tableKey="name" />
        <template v-for="row in rows" :key="row[0]">
            <TableRow :row="row[1]" :columns="columns" />
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
    border-bottom: 1px solid var(--color-border);
}
</style>
