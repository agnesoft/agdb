<script lang="ts" setup>
import { defineProps, computed, type PropType } from "vue";
import { type TRow } from "@/composables/table/tableConfig";

const props = defineProps({
    row: {
        type: Object as PropType<TRow>,
        required: true,
    },
});
const cellKeys = computed(() => {
    return Object.keys(props.row);
});
</script>

<template>
    <div class="agdb-table-row columns">
        <div v-for="cellKey in cellKeys" :key="cellKey">
            <p>{{ row[cellKey] }}</p>
        </div>
    </div>
</template>

<style lang="less" scoped></style>
