<script lang="ts" setup>
import { computed, defineProps } from "vue";
import { getRows } from "@/composables/table/tableData";
import TableRow from "@/components/base/table/TableRow.vue";
import TableHeader from "./TableHeader.vue";
import { getTableColumns, type TRow } from "@/composables/table/tableConfig";

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
</script>

<template>
    <div class="agdb-table">
        <TableHeader :tableKey="name" />
        <template v-for="row in rows" :key="row[0]">
            <TableRow v-if="columns" :row="row[1]" :columns="columns" />
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
