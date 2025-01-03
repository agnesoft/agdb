<script lang="ts" setup>
import AgdbTable from "../base/table/AgdbTable.vue";
import { useDbStore } from "@/composables/db/dbStore";
import { addTable } from "@/composables/table/tableConfig";
import { setTableData } from "@/composables/table/tableData";
import { watchEffect } from "vue";
import { dbColumns } from "@/composables/db/dbConfig";

const { databases } = useDbStore();

const TABLE_KEY = "databases";

addTable({
    name: TABLE_KEY,
    columns: dbColumns,
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
    width: 1300px;
    margin: 0 auto;
}
</style>
