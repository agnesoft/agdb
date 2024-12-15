<script lang="ts" setup>
import AgdbTable from "../base/table/AgdbTable.vue";
import { useDbList } from "@/composables/stores/DbStore";
import { addTable } from "@/composables/table/tableConfig";
import { setTableData } from "@/composables/table/tableData";
import { watchEffect } from "vue";

const { databases } = useDbList();

const TABLE_KEY = "databases";

addTable(TABLE_KEY, [
    { key: "role", title: "Role" },
    { key: "name", title: "Name" },
    { key: "db_type", title: "Type" },
    { key: "size", title: "Size" },
]);

watchEffect(() => {
    setTableData(TABLE_KEY, databases.value);
});
</script>

<template>
    <div>
        <div v-if="databases.length" class="table-wrap">
            <AgdbTable :name="TABLE_KEY" />
        </div>
        <p v-else>No databases found</p>
    </div>
</template>

<style lang="less" scoped>
.table-wrap {
    width: 800px;
    margin: 0 auto;
    max-width: 100%;
}
</style>
