<script lang="ts" setup>
import AgdbTable from "../base/table/AgdbTable.vue";
import { useDbStore } from "@/composables/db/DbStore";
import { addTable } from "@/composables/table/tableConfig";
import { setTableData } from "@/composables/table/tableData";
import { watchEffect } from "vue";
import { dateFormatter } from "@/composables/table/utils";
import dbActions from "@/composables/db/DbActions";

const { databases } = useDbStore();

const TABLE_KEY = "databases";

addTable({
    name: TABLE_KEY,
    columns: [
        { key: "role", title: "Role" },
        { key: "owner", title: "Owner" },
        { key: "db", title: "Name" },
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
            actions: dbActions,
        },
    ],
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
    width: 1200px;
    margin: 0 auto;
}
</style>
