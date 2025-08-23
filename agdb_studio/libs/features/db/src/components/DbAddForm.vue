<script lang="ts" setup>
import { ref } from "vue";
import { useDbStore } from "../composables/dbStore";
import type { DbType } from "@agnesoft/agdb_api/openapi";

const name = ref("");
const db_type = ref<DbType>("memory");

const { addDatabase, fetchDatabases } = useDbStore();

const loading = ref(false);

const add = (event: Event) => {
  if (loading.value) return;
  if (!name.value) return;
  loading.value = true;
  event.preventDefault();

  addDatabase({
    name: name.value,
    db_type: db_type.value,
  })
    .then(() => {
      loading.value = false;
      name.value = "";
      db_type.value = "memory";
      fetchDatabases();
    })
    .catch(() => {
      loading.value = false;
    });
};
</script>

<template>
  <div class="db-add-form">
    <h2>Add Database</h2>
    <form id="db-add-form" @submit="add">
      <input
        v-model="name"
        type="text"
        placeholder="Name"
        name="db-name"
        data-testid="db-name-input"
        required
      />
      <select v-model="db_type" name="db-type">
        <option value="memory">Memory</option>
        <option value="file">File</option>
        <option value="mapped">Mapped</option>
      </select>
      <button
        type="submit"
        class="button"
        @click="add"
        data-testid="add-db-button"
      >
        Add
      </button>
    </form>
  </div>
</template>

<style lang="less" scoped>
.db-add-form {
  margin: 1rem auto;
  padding: 1rem;
  border: 1px solid var(--color-border);
  border-radius: 0.5rem;
}

input,
select {
  margin: 0.3rem 0.3rem 0 0;
  padding: 0.2rem 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 0.2rem;
}
</style>
