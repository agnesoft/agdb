<script lang="ts" setup>
import { computed, defineProps } from "vue";
import { useTableData } from "@/composables/table/tableData";
import TableRow from "@/components/base/table/TableRow.vue";

const props = defineProps({
    name: {
        type: String,
        required: true,
    },
});

const { getRows } = useTableData();
const rows = computed(() => {
    return getRows(props.name);
});
</script>

<template>
    <div class="agdb-table">
        <div v-for="row in rows" :key="row[0]">
            <TableRow :row="row[1]" />
        </div>
    </div>
</template>

<style lang="less" scoped></style>
