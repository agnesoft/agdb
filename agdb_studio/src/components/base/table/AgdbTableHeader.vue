<script lang="ts" setup>
import { getTableColumnsArray } from "@/composables/table/tableConfig";
import { computed } from "vue";
const props = defineProps({
    tableKey: {
        type: String,
        required: true,
    },
});

const columns = computed(() => {
    return getTableColumnsArray(props.tableKey);
});
</script>

<template>
    <div class="agdb-table-header columns">
        <div v-for="column in columns" :key="column.key">
            {{ column.title }}
        </div>
    </div>
</template>

<style lang="less" scoped>
.agdb-table-header {
    &.columns {
        border: 1px solid var(--color-border);
    }

    div {
        font-weight: bold;
        font-size: 1.05rem;
    }
}
</style>
