<script lang="ts" setup>
import { addTable } from "@agdb-studio/common/src/composables/table/tableConfig";
import { setTableData } from "@agdb-studio/common/src/composables/table/tableData";
import { userColumns } from "../composables/userConfig";
import { useUserStore } from "../composables/userStore";
import { watchEffect } from "vue";
import AgdbTable from "@agdb-studio/common/src/components/table/AgdbTable.vue";

const { users, fetchUsers } = useUserStore();

const TABLE_KEY = Symbol("users");

addTable({
  name: TABLE_KEY,
  columns: userColumns,
  uniqueKey: "username",
  fetchData: fetchUsers,
});

watchEffect(() => {
  setTableData(TABLE_KEY, users.value);
});
</script>

<template>
  <div class="table-wrap">
    <div v-if="users.length" class="user-table">
      <AgdbTable :name="TABLE_KEY" />
    </div>

    <p v-else>No users found</p>
  </div>
</template>

<style lang="less" scoped>
.table-wrap {
  overflow: auto;
}
.user-table {
  width: 700px;
  margin: 0 auto;
}
</style>
