<script lang="ts" setup>
import type { ServerDatabase } from "agdb_api/dist/openapi";
import { defineProps, type PropType } from "vue";

defineProps({
    db: {
        type: Object as PropType<ServerDatabase>,
        required: true,
    },
});
</script>

<template>
    <div class="db-list-row">
        <div>{{ db.role }}</div>
        <div>{{ db.name }}</div>
        <div>{{ db.db_type }}</div>
        <div>{{ db.size }}</div>
    </div>
</template>

<style lang="less" scoped>
.db-list-row {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 1rem;
    padding: 0.5rem;
    border-bottom: 1px solid var(--color-border);
    margin: 0 auto;
}
</style>
