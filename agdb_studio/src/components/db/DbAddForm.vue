<script lang="ts" setup>
import { ref } from "vue";
import { useDbList } from "@/composables/stores/DbStore";
import type { DbType } from "agdb_api/dist/openapi";

const name = ref("");
const db_type = ref<DbType>("memory");

const { addDatabase } = useDbList();

const add = () => {
    addDatabase({
        name: name.value,
        db_type: db_type.value,
    });
};
</script>

<template>
    <div class="db-add-form">
        <h2>Add Database</h2>
        <form @submit="add">
            <input type="text" v-model="name" placeholder="Name" />
            <select v-model="db_type">
                <option value="memory">Memory</option>
                <option value="file">File</option>
                <option value="mapped">Mapped</option>
            </select>
            <button type="submit" class="button" @click="add">Add</button>
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
