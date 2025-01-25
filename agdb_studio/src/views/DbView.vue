<script lang="ts" setup>
import { useDbStore } from "@/composables/db/dbStore";
import { onMounted } from "vue";
import DbAddForm from "@/components/db/DbAddForm.vue";
import DbTable from "@/components/db/DbTable.vue";
import { JaRefreshReverse } from "@kalimahapps/vue-icons";

const { fetchDatabases } = useDbStore();
onMounted(async () => {
    await fetchDatabases();
});
</script>

<template>
    <div class="db-view">
        <div class="header">
            <DbAddForm />
            <button
                class="button refresh"
                @click="fetchDatabases"
                title="refresh"
            >
                <JaRefreshReverse />
            </button>
        </div>
        <DbTable />
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
    .button {
        grid-area: refresh;
    }
}
</style>
