<script lang="ts" setup>
import UserAddForm from "@agdb-studio/user/src/components/UserAddForm.vue";
import UserTable from "@agdb-studio/user/src/components/UserTable.vue";
import { useUserStore } from "@agdb-studio/user/src/composables/userStore";
import { logoutAllUsers } from "@agdb-studio/user/src/composables/userConfig";
import SpinnerIcon from "@agdb-studio/design/src/components/icons/SpinnerIcon.vue";
import { MdRefresh } from "@kalimahapps/vue-icons";
import { onMounted, ref } from "vue";

const { fetchUsers } = useUserStore();

const loading = ref(true);

onMounted(async () => {
  try {
    await fetchUsers();
  } catch {
    // API error interceptor handles notification
  } finally {
    loading.value = false;
  }
});

const refresh = async () => {
  loading.value = true;
  try {
    await fetchUsers();
  } catch {
    // API error interceptor handles notification
  } finally {
    loading.value = false;
  }
};
</script>

<template>
  <div class="admin-user-view">
    <UserAddForm />
    <div class="toolbar">
      <button
        class="button button-danger logout-all"
        title="Logout all users"
        aria-label="Logout all users"
        data-testid="logout-all-button"
        @click="logoutAllUsers"
      >
        Logout All
      </button>
      <button
        class="button refresh"
        title="refresh"
        aria-label="Refresh users"
        data-testid="refresh-button"
        @click="refresh"
      >
        <MdRefresh />
      </button>
    </div>
    <SpinnerIcon v-if="loading" />
    <UserTable v-else class="table" />
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
    "table toolbar"
    "table .";
  max-width: 1000px;
  justify-items: start;
  align-items: start;
  margin: 0 auto;
  .toolbar {
    grid-area: toolbar;
    display: flex;
    gap: 0.5rem;
    align-items: center;
    justify-content: space-between;
    width: 100%;
  }
  .refresh {
    width: 2rem;
    height: 2rem;
    font-size: 1rem;
    padding: 0;
    display: flex;
    justify-content: center;
    align-items: center;
  }
  .logout-all {
    font-size: 0.75rem;
    padding: 0.25rem 0.5rem;
  }
  .user-add-form {
    grid-area: form;
  }
  .table {
    grid-area: table;
  }
}
</style>
