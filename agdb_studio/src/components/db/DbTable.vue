<script lang="ts" setup>
import AgdbTable from "../base/table/AgdbTable.vue";
import { useDbList } from "@/composables/stores/DbStore";
import { addTable } from "@/composables/table/tableConfig";
import { setTableData } from "@/composables/table/tableData";
import { watchEffect } from "vue";
import { dateFormatter } from "@/composables/table/utils";

const { databases } = useDbList();

const TABLE_KEY = "databases";

addTable({
    name: TABLE_KEY,
    columns: [
        { key: "role", title: "Role" },
        { key: "name", title: "Owner/Name" },
        { key: "db_type", title: "Type" },
        { key: "size", title: "Size" },
        {
            key: "backup",
            title: "Backup",
            valueFormatter: dateFormatter,
        },
        {
            key: "actions",
            title: "Actions",
            actions: [
                { label: "Edit", action: () => console.log("Edit") },
                { label: "Delete", action: () => console.log("Delete") },
            ],
        },
    ],
    uniqueKey: "name",
});

watchEffect(() => {
    setTableData(TABLE_KEY, databases.value);
});
</script>

<template>
    <div class="table-wrap">
        <div v-if="databases.length" class="db-table">
            <AgdbTable :name="TABLE_KEY" />
        </div>

        <p v-else>No databases found</p>
    </div>
</template>

<style lang="less" scoped>
.table-wrap {
    overflow: auto;
}
.db-table {
    width: 1100px;
    margin: 0 auto;
}
</style>
