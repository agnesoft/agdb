<script lang="ts" setup>
import { useDbList } from "@/composables/stores/DbStore";
import { onMounted } from "vue";
import DbAddForm from "./DbAddForm.vue";
import DbTable from "./DbTable.vue";

const { fetchDatabases } = useDbList();
onMounted(async () => {
    await fetchDatabases();
});
</script>

<template>
    <div class="db-list">
        <button class="button" @click="fetchDatabases">Refresh</button>
        <DbAddForm />
        <DbTable />
    </div>
</template>

<style lang="less" scoped>
.db-list {
    text-align: center;
}
</style>
