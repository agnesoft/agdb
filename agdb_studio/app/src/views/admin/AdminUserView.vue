<script lang="ts" setup>
import UserAddForm from "@agdb-studio/user/src/components/UserAddForm.vue";
import UserTable from "@agdb-studio/user/src/components/UserTable.vue";
import { useUserStore } from "@agdb-studio/user/src/composables/userStore";
import { MdRefresh } from "@kalimahapps/vue-icons";
import { onMounted } from "vue";

const { fetchUsers } = useUserStore();
onMounted(async () => {
  await fetchUsers();
});
</script>

<template>
  <div class="admin-user-view">
    <UserAddForm />
    <button class="button refresh" title="refresh" @click="fetchUsers">
      <MdRefresh />
    </button>
    <UserTable class="table" />
  </div>
</template>

<style lang="less" scoped>
.admin-user-view {
  text-align: center;
  display: grid;
  grid-template-columns: 1fr max-content;
  grid-template-rows: max-content max-content 1fr;
  grid-template-areas:
    "table form"
    "table refresh"
    "table .";
  max-width: 1000px;
  justify-items: start;
  align-items: start;
  margin: 0 auto;
  .button {
    width: 2rem;
    height: 2rem;
    font-size: 1rem;
    padding: 0;
    display: flex;
    justify-content: center;
    align-items: center;
  }
  .user-add-form {
    grid-area: form;
  }
  .button {
    grid-area: refresh;
  }
  .table {
    grid-area: table;
  }
}
</style>
