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

const column = computed(() => columns?.value.get(props.cellKey));
const value = computed(() => {
  return row?.value[props.cellKey] ?? "";
});

const formattedValue = computed(() => {
  if (column.value?.valueFormatter) {
    return column.value.valueFormatter(value.value);
  }
  return value.value;
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
    <div v-else>
      <p>{{ formattedValue }}</p>
    </div>
  </div>
</template>

<style lang="less" scoped>
.positive-icon {
  color: var(--success-color-2);
}
.negative-icon {
  color: var(--error-color-2);
}
</style>
