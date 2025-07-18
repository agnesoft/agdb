<script lang="ts" setup>
import { computed, type PropType, provide, ref, useSlots } from "vue";
import { type Column, type TRow } from "../../composables/table/types";
import { INJECT_KEY_ROW } from "../../composables/table/constants";
import AgdbCell from "./AgdbCell.vue";
import { AkChevronDownSmall, AkChevronUpSmall } from "@kalimahapps/vue-icons";
import SlideUpTransition from "@agdb-studio/design/src/components/transitions/SlideUpTransition.vue";

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

const rowExpanded = ref(false);
const toggleExpandRow = (): void => {
  rowExpanded.value = !rowExpanded.value;
};

const slots = useSlots();
</script>

<template>
  <div class="agdb-table-row-wrap">
    <div :class="['agdb-table-row columns', { expandable: slots.rowDetails }]">
      <div v-for="cellKey in cellKeys" :key="cellKey">
        <AgdbCell :cell-key="cellKey" />
      </div>
      <div v-if="slots.rowDetails">
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
      <div v-if="rowExpanded && slots.rowDetails" class="expanded-row">
        <slot name="rowDetails" :row="rowData"></slot>
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
