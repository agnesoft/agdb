<script lang="ts" setup>
import type { Components } from "@agnesoft/agdb_api";
import { computed } from "vue";
import { dbValueToString } from "../../utils/graphDataUtils";

const props = defineProps<{
  results: Components.Schemas.QueryResult[];
}>();

const emit = defineEmits<{
  menu: [element: Components.Schemas.DbElement, x: number, y: number];
}>();

const elements = computed(() => props.results.flatMap((r) => r.elements));

const hasEdges = computed(() => elements.value.some((el) => el.id < 0));

/** Union of all keys in order of first appearance. */
const keys = computed(() => {
  const seen = new Set<string>();
  for (const el of elements.value) {
    for (const kv of el.values) seen.add(dbValueToString(kv.key));
  }
  return [...seen];
});

const rows = computed(() =>
  elements.value.map((el) => {
    const values = new Map(
      el.values.map((kv) => [
        dbValueToString(kv.key),
        dbValueToString(kv.value),
      ]),
    );
    return { element: el, cells: keys.value.map((k) => values.get(k) ?? "") };
  }),
);

const openMenu = (element: Components.Schemas.DbElement, event: MouseEvent) => {
  emit("menu", element, event.clientX, event.clientY);
};
</script>

<template>
  <div class="results-table" data-testid="results-table">
    <table v-if="rows.length">
      <thead>
        <tr>
          <th class="actions-col"></th>
          <th>id</th>
          <template v-if="hasEdges">
            <th>from</th>
            <th>to</th>
          </template>
          <th v-for="key in keys" :key="key">{{ key }}</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="(row, index) in rows"
          :key="index"
          data-testid="results-table-row"
          @contextmenu.prevent="
            (event: MouseEvent) =>
              row.element.id !== 0 && openMenu(row.element, event)
          "
        >
          <td class="actions-col">
            <button
              v-if="row.element.id !== 0"
              type="button"
              class="button row-menu"
              title="Element actions"
              data-testid="results-row-menu"
              @click.stop="(event: MouseEvent) => openMenu(row.element, event)"
            >
              ⋮
            </button>
          </td>
          <td class="id">
            {{ row.element.id === 0 ? "(database)" : row.element.id }}
          </td>
          <template v-if="hasEdges">
            <td>{{ row.element.id < 0 ? row.element.from : "" }}</td>
            <td>{{ row.element.id < 0 ? row.element.to : "" }}</td>
          </template>
          <td v-for="(cell, cellIndex) in row.cells" :key="cellIndex">
            {{ cell }}
          </td>
        </tr>
      </tbody>
    </table>
    <p v-else class="empty">No elements.</p>
  </div>
</template>

<style lang="less" scoped>
.results-table {
  height: 100%;
  min-height: 20rem;
  max-height: calc(100vh - 14rem);
  overflow: auto;
  border: 1px solid var(--color-border);
  border-radius: 4px;
}
table {
  width: 100%;
  border-collapse: collapse;
}
th,
td {
  text-align: left;
  padding: 0.35rem 0.6rem;
  border-bottom: 1px solid var(--color-border);
  white-space: nowrap;
}
th {
  position: sticky;
  top: 0;
  background-color: var(--color-background-mute);
  color: var(--color-text-muted);
  font-size: 0.85rem;
}
tbody tr:hover {
  background-color: var(--color-background-soft);
}
.id {
  font-weight: bold;
}
.actions-col {
  width: 2rem;
}
.row-menu {
  padding: 0 0.4rem;
}
.empty {
  padding: 1rem;
  color: var(--color-text-muted);
}
</style>
