<script lang="ts" setup>
import type { Components } from "@agnesoft/agdb_api";
import { computed } from "vue";
import {
  dbValueToString,
  dedupeResultElements,
} from "../../utils/graphDataUtils";

const props = defineProps<{
  results: Components.Schemas.QueryResult[];
  /** Alias of a node id, shown in its own column when any node has one. */
  aliasOf?: (id: number) => string | undefined;
}>();

const emit = defineEmits<{
  menu: [element: Components.Schemas.DbElement, x: number, y: number];
}>();

// An element several queries returned is one row; the JSON view keeps them all.
const deduped = computed(() => dedupeResultElements(props.results));
const elements = computed(() => deduped.value.map((d) => d.element));

const aliases = computed(
  () => new Map(elements.value.map((el) => [el.id, props.aliasOf?.(el.id)])),
);
const hasAliases = computed(() =>
  [...aliases.value.values()].some((alias) => alias !== undefined),
);
// Which queries returned a row matters only for batches.
const showOrigin = computed(() => props.results.length > 1);

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
  deduped.value.map(({ element: el, resultIndexes }, index) => {
    const values = new Map(
      el.values.map((kv) => [
        dbValueToString(kv.key),
        dbValueToString(kv.value),
      ]),
    );
    return {
      // Id 0 rows (not elements) can repeat.
      key: el.id === 0 ? `0:${index}` : el.id,
      element: el,
      alias: aliases.value.get(el.id) ?? "",
      origin: resultIndexes.map((i) => `#${i}`).join(", "),
      cells: keys.value.map((k) => values.get(k) ?? ""),
    };
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
          <th v-if="hasAliases">alias</th>
          <th v-if="showOrigin" title="Queries of the batch that returned it">
            result
          </th>
          <template v-if="hasEdges">
            <th>from</th>
            <th>to</th>
          </template>
          <th v-for="key in keys" :key="key">{{ key }}</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="row in rows"
          :key="row.key"
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
              aria-label="Element actions"
              data-testid="results-row-menu"
              @click.stop="(event: MouseEvent) => openMenu(row.element, event)"
            >
              ⋮
            </button>
          </td>
          <td class="id">
            {{ row.element.id === 0 ? "(database)" : row.element.id }}
          </td>
          <td v-if="hasAliases" class="alias">{{ row.alias }}</td>
          <td v-if="showOrigin" class="origin">{{ row.origin }}</td>
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
.alias,
.origin {
  color: var(--color-text-muted);
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
