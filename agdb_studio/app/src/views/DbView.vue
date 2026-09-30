<script lang="ts" setup>
import { useDbStore } from "@agdb-studio/db/src/composables/dbStore";
import { onMounted, ref } from "vue";
import DbAddForm from "@agdb-studio/db/src/components/DbAddForm.vue";
import DbTable from "@agdb-studio/db/src/components/DbTable.vue";
import SpinnerIcon from "@agdb-studio/design/src/components/icons/SpinnerIcon.vue";
import { MdRefresh } from "@kalimahapps/vue-icons";

const { fetchDatabases } = useDbStore();

const loading = ref(true);

onMounted(async () => {
  await fetchDatabases();
  loading.value = false;
});

const refresh = async () => {
  loading.value = true;
  await fetchDatabases();
  loading.value = false;
};
</script>

<template>
  <div class="db-view">
    <div class="header">
      <DbAddForm />
      <button
        class="button refresh"
        title="refresh"
        aria-label="Refresh databases"
        data-testid="refresh-button"
        @click="refresh"
      >
        <MdRefresh />
      </button>
    </div>
    <SpinnerIcon v-if="loading" />
    <DbTable v-else />
  </div>
</template>

<style lang="less" scoped>
.db-view {
  text-align: center;
}
.header {
  display: grid;
  justify-content: space-between;
  align-items: center;
  grid-template-columns: 2rem 1fr 2rem;
  grid-template-areas: ". form refresh";
  max-width: 1200px;
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
  .db-add-form {
    grid-area: form;
  }
}
</style>
