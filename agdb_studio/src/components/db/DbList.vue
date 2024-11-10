<script lang="ts" setup>
import { useDbList } from "@/composables/DbStore";
import { onMounted } from "vue";
import DbListRow from "./DbListRow.vue";
import DbAddForm from "./DbAddForm.vue";

const { fetchDatabases, databases } = useDbList();
onMounted(async () => {
    await fetchDatabases();
});
</script>

<template>
    <div class="db-list">
        <button class="button" @click="fetchDatabases">Refresh</button>
        <DbAddForm />
        <ul v-if="databases.length">
            <li v-for="db in databases" :key="db.name">
                <DbListRow :db="db" />
            </li>
        </ul>
        <p v-else>No databases found</p>
    </div>
</template>

<style lang="less" scoped>
.db-list {
    text-align: center;
}
</style>
