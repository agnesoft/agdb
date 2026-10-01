<script lang="ts" setup>
import UserAddForm from "@agdb-studio/user/src/components/UserAddForm.vue";
import UserTable from "@agdb-studio/user/src/components/UserTable.vue";
import { useUserStore } from "@agdb-studio/user/src/composables/userStore";
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
    <button
      class="button refresh"
      title="refresh"
      aria-label="Refresh users"
      data-testid="refresh-button"
      @click="refresh"
    >
      <MdRefresh />
    </button>
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
    "table refresh"
    "table .";
  max-width: 1000px;
  justify-items: start;
  align-items: start;
  margin: 0 auto;
  .button {
    grid-area: refresh;
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
  .table {
    grid-area: table;
  }
}
</style>
