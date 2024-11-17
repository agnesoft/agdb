<script lang="ts" setup>
import AgdbTable from "../base/table/AgdbTable.vue";
import { useDbList } from "@/composables/stores/DbStore";
import { useTableConfig } from "@/composables/table/tableConfig";
import { useTableData } from "@/composables/table/tableData";
import { watchEffect } from "vue";

const { databases } = useDbList();

const { addTable } = useTableConfig();

const TABLE_KEY = "databases";

addTable(TABLE_KEY, [
    { key: "role", title: "Role" },
    { key: "name", title: "Name" },
    { key: "db_type", title: "Type" },
    { key: "size", title: "Size" },
]);

const { addRow } = useTableData();
watchEffect(() => {
    databases.value.forEach((db) => {
        addRow(TABLE_KEY, db.name, db);
    });
});
</script>

<template>
    <div>
        <div v-if="databases.length">
            <AgdbTable :name="TABLE_KEY" />
        </div>
        <p v-else>No databases found</p>
    </div>
</template>

<style lang="less" scoped></style>
