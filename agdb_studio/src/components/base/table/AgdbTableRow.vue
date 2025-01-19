<script lang="ts" setup>
import { computed, inject, type PropType, provide, type Ref, ref } from "vue";
import { type Column, type TRow } from "@/composables/table/types";
import {
    INJECT_KEY_ROW,
    INJECT_KEY_TABLE_NAME,
} from "@/composables/table/constants";
import AgdbCell from "./AgdbCell.vue";
import { getTable } from "@/composables/table/tableConfig";
import { AkChevronDownSmall, AkChevronUpSmall } from "@kalimahapps/vue-icons";
import SlideUpTransition from "@/components/transitions/SlideUpTransition.vue";
import { getAsyncComponent } from "@/utils/asyncComponents";

const props = defineProps({
    row: {
        type: Object as PropType<TRow>,
        required: true,
    },
    columns: {
        type: Object as PropType<Map<string, Column<TRow>>>,
        required: true,
    },
});
const cellKeys = computed(() => {
    return Object.keys(props.row);
});

const rowData = computed(() => {
    return props.row;
});
provide(INJECT_KEY_ROW, rowData);

const tableKey = inject<Ref<Symbol | string>>(INJECT_KEY_TABLE_NAME);
const rowDetailsComponent = computed(() => {
    const name = tableKey
        ? getTable(tableKey.value)?.rowDetailsComponent
        : undefined;
    if (name) {
        return getAsyncComponent(name);
    }
    return undefined;
});
const rowExpanded = ref(false);
const toggleExpandRow = (): void => {
    rowExpanded.value = !rowExpanded.value;
};
</script>

<template>
    <div class="agdb-table-row-wrap">
        <div
            :class="[
                'agdb-table-row columns',
                { expandable: rowDetailsComponent },
            ]"
        >
            <div v-for="cellKey in cellKeys" :key="cellKey">
                <AgdbCell :cell-key="cellKey" />
            </div>
            <div v-if="rowDetailsComponent">
                <button
                    class="button button-transparent expand-row"
                    @click="toggleExpandRow"
                >
                    <AkChevronDownSmall v-if="!rowExpanded" />
                    <AkChevronUpSmall v-else />
                </button>
            </div>
        </div>
        <SlideUpTransition>
            <div v-if="rowExpanded && rowDetailsComponent" class="expanded-row">
                <component :is="rowDetailsComponent" :row="row" />
            </div>
        </SlideUpTransition>
    </div>
</template>

<style lang="less" scoped>
.agdb-table-row-wrap {
    border-bottom: 1px solid var(--color-border);
    .expanded-row {
        border: 1px solid var(--color-border);
        border-bottom: none;
    }
}
</style>
