<script lang="ts" setup>
import { INJECT_KEY_TABLE_NAME } from "../../composables/table/constants";
import { getTableColumnsArray } from "../../composables/table/tableConfig";
import { computed, inject, type Ref } from "vue";

const tableKey = inject<Ref<symbol | string>>(INJECT_KEY_TABLE_NAME);
const columns = computed(() => {
  if (!tableKey?.value) {
    return [];
  }
  return getTableColumnsArray(tableKey.value);
});
defineProps({
  rowsExpandable: {
    type: Boolean,
    default: false,
  },
});
</script>

<template>
  <div :class="['agdb-table-header columns', { expandable: rowsExpandable }]">
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
