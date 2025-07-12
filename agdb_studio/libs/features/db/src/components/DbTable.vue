<script lang="ts" setup>
import AgdbTable from "@agdb-studio/common/src/components/table/AgdbTable.vue";
import { useDbStore } from "../composables/dbStore";
import { addTable } from "@agdb-studio/common/src/composables/table/tableConfig";
import { setTableData } from "@agdb-studio/common/src/composables/table/tableData";
import { watchEffect, provide } from "vue";
import { dbColumns } from "../composables/dbConfig";
import DbDetails from "./DbDetails.vue";
// import { getAsyncComponent } from "@/utils/asyncComponents";

const { databases, getDbName, fetchDatabases } = useDbStore();

const TABLE_KEY = Symbol("databases");

// provide("getAsyncComponent", getAsyncComponent);

addTable({
  name: TABLE_KEY,
  columns: dbColumns,
  // rowDetailsComponent: "DbDetails",
  uniqueKey: (row) =>
    getDbName({ owner: row.owner.toString(), db: row.db.toString() }),
  fetchData: fetchDatabases,
});

watchEffect(() => {
  setTableData(TABLE_KEY, databases.value);
});
</script>

<template>
  <div class="table-wrap">
    <div v-if="databases.length" class="db-table">
      <AgdbTable :name="TABLE_KEY">
        <template #rowDetails="{ row }">
          <DbDetails :db="row" />
        </template>
      </AgdbTable>
    </div>

    <p v-else>No databases found</p>
  </div>
</template>

<style lang="less" scoped>
.table-wrap {
  overflow: auto;
}
.db-table {
  width: 1400px;
  margin: 0 auto;
}
</style>
