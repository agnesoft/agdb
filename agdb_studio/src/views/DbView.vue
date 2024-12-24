<script lang="ts" setup>
import { useDbStore } from "@/composables/stores/DbStore";
import { onMounted } from "vue";
import DbAddForm from "@/components/db/DbAddForm.vue";
import DbTable from "@/components/db/DbTable.vue";

const { fetchDatabases } = useDbStore();
onMounted(async () => {
    await fetchDatabases();
});
</script>

<template>
    <div class="db-view">
        <button class="button" @click="fetchDatabases">Refresh</button>
        <DbAddForm />
        <DbTable />
    </div>
</template>

<style lang="less" scoped>
.db-view {
    text-align: center;
}
</style>
