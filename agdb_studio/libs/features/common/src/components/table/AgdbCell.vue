<script lang="ts" setup>
import {
  INJECT_KEY_COLUMNS,
  INJECT_KEY_ROW,
} from "../../composables/table/constants";
import type { Column, TRow } from "../../composables/table/types";
import { computed, inject, type Ref } from "vue";
import AgdbCellMenu from "./AgdbCellMenu.vue";
import { BsCheckLg, ClCloseMd } from "@kalimahapps/vue-icons";

const props = defineProps({
  cellKey: {
    type: String,
    required: true,
  },
});
const columns = inject<Ref<Map<string, Column<TRow>>>>(INJECT_KEY_COLUMNS);
const row = inject<Ref<TRow>>(INJECT_KEY_ROW);
const rowValue = computed<TRow>(() => row?.value ?? ({} as TRow));

const column = computed(() => columns?.value.get(props.cellKey));
const value = computed(() => {
  return rowValue.value[props.cellKey] ?? "";
});

const formattedValue = computed(() => {
  if (column.value?.valueFormatter) {
    return column.value.valueFormatter(value.value);
  }
  return value.value;
});

const leadingIcon = computed(() => {
  if (column.value?.iconResolver) {
    return column.value.iconResolver(rowValue.value);
  }

  if (!column.value?.icon) {
    return undefined;
  }

  return column.value.icon;
});

const leadingIconTitle = computed(() => {
  if (column.value?.iconTitleResolver) {
    return column.value.iconTitleResolver(rowValue.value);
  }

  if (!column.value?.iconTitle) {
    return undefined;
  }

  return column.value.iconTitle;
});

const leadingIconClass = computed(() => {
  if (column.value?.iconClassResolver) {
    return column.value.iconClassResolver(rowValue.value);
  }

  if (!column.value?.iconClass) {
    return undefined;
  }

  return column.value.iconClass;
});
</script>

<template>
  <div class="agdb-cell">
    <div v-if="column?.actions">
      <AgdbCellMenu :actions="column.actions" />
    </div>
    <div v-else-if="column?.type === 'boolean'">
      <BsCheckLg v-if="value" class="positive-icon" title="Yes" />
      <ClCloseMd v-else class="negative-icon" title="No" />
    </div>
    <div v-else class="cell-content">
      <p>{{ formattedValue }}</p>
      <component
        :is="leadingIcon"
        v-if="leadingIcon"
        :class="leadingIconClass"
        :title="leadingIconTitle"
      />
    </div>
  </div>
</template>

<style lang="less" scoped>
.cell-content {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
}

.positive-icon {
  color: var(--success-color-2);
}
.negative-icon {
  color: var(--error-color-2);
}
</style>
