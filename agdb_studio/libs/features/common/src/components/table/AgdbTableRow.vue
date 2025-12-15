<script lang="ts" setup>
import {
  computed,
  inject,
  type PropType,
  provide,
  type Ref,
  ref,
  useSlots,
} from "vue";
import { type Column, type TRow } from "../../composables/table/types";
import {
  INJECT_KEY_ROW,
  INJECT_KEY_TABLE_NAME,
} from "../../composables/table/constants";
import AgdbCell from "./AgdbCell.vue";
import { AkChevronDownSmall, AkChevronUpSmall } from "@kalimahapps/vue-icons";
import SlideUpTransition from "@agdb-studio/design/src/components/transitions/SlideUpTransition.vue";
import { getTableRowClickHandler } from "../../composables/table/tableConfig";

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

const tableKey = inject<Ref<symbol | string>>(INJECT_KEY_TABLE_NAME);

const onRowClick = computed(() =>
  tableKey?.value ? getTableRowClickHandler<TRow>(tableKey.value) : undefined,
);

const handleRowClick = (): void => {
  /* v8 ignore else -- @preserve */
  if (onRowClick.value) {
    onRowClick.value(props.row);
  }
};
</script>

<template>
  <div class="agdb-table-row-wrap">
    <div
      :class="[
        'agdb-table-row columns',
        { expandable: slots.rowDetails, clickable: !!onRowClick },
      ]"
      @click="handleRowClick"
    >
      <div
        v-for="cellKey in cellKeys"
        :key="cellKey"
        :data-testid="`table-cell-${cellKey}`"
      >
        <AgdbCell :cell-key="cellKey" />
      </div>
      <div v-if="slots.rowDetails">
        <button
          class="button button-transparent expand-row"
          :aria-expanded="rowExpanded"
          :title="rowExpanded ? 'Collapse row details' : 'Expand row details'"
          @click.stop="toggleExpandRow"
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
}
.expanded-row {
  border: 1px solid var(--color-border);
  border-bottom: none;
}
.clickable {
  cursor: pointer;
  transition: background-color 0.2s ease-in-out;
  // hover effect on clickable rows but not on buttons inside the row
  &:not(&:hover:has(button:hover)):hover {
    background-color: var(--color-background-soft);
  }
}
.expand-row {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  padding: 0;
}
</style>
