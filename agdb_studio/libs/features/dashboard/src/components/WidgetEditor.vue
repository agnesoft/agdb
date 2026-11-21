<script lang="ts" setup>
import { computed, onUnmounted, ref } from "vue";
import QueryView from "@agdb-studio/query/src/components/view/QueryView.vue";
import { provideDbTarget } from "@agdb-studio/query/src/composables/dbExec";
import { useQueryStore } from "@agdb-studio/query/src/composables/queryStore";
import { useDbStore } from "@agdb-studio/db/src/composables/dbStore";
import { WIDGET_TABS, type Widget } from "../types";
import {
  MAX_REFRESH_SECONDS,
  MIN_REFRESH_SECONDS,
  isRefreshInterval,
} from "../utils/refresh";
import {
  hasUnfinishedRow,
  seedQuery,
  widgetFromQuery,
} from "../utils/widgetRows";

const props = defineProps<{
  widget: Widget;
  header?: string;
}>();

const emit = defineEmits<{
  done: [widget: Widget];
  cancel: [];
}>();

const { databases } = useDbStore();
const queryStore = useQueryStore();

const title = ref(props.widget.title);
const database = ref(
  props.widget.owner
    ? `${props.widget.owner}/${props.widget.db}`
    : (databases.value.map((d) => `${d.owner}/${d.db}`)[0] ?? ""),
);
const refresh = ref(String(props.widget.refreshSeconds ?? 0));

const target = computed(() => {
  const [owner = "", ...db] = database.value.split("/");
  return { owner, db: db.join("/") };
});

const databaseOptions = computed(() => {
  const names = databases.value.map((d) => `${d.owner}/${d.db}`);
  // Keep the widget's database even when it is not listed (e.g. no access).
  return database.value && !names.includes(database.value)
    ? [database.value, ...names]
    : names;
});

// The builder and results work on their own query entry, on the chosen database.
const queryId = `widget-editor-${crypto.randomUUID()}`;
seedQuery(queryId, props.widget);
provideDbTarget(target);
onUnmounted(() => queryStore.deleteQuery(queryId));

const hasQuery = computed(
  () =>
    !!queryStore
      .getQuery(queryId)
      ?.value.rows.exec.some((row) => row.steps.length > 0),
);

const refreshSeconds = computed(() => Number(refresh.value) || 0);
// 0 turns auto refresh off; anything else must be a sane interval.
const refreshError = computed(() =>
  refreshSeconds.value !== 0 && !isRefreshInterval(refreshSeconds.value)
    ? `0 (off) or ${MIN_REFRESH_SECONDS} to ${MAX_REFRESH_SECONDS} seconds`
    : undefined,
);

const unfinished = computed(() => {
  const query = queryStore.getQuery(queryId)?.value;
  return !!query && hasUnfinishedRow(query);
});
const canSave = computed(
  () =>
    hasQuery.value &&
    !unfinished.value &&
    !!target.value.db &&
    !refreshError.value,
);

const done = () => {
  const query = queryStore.getQuery(queryId)?.value;
  /* v8 ignore next -- @preserve: the entry lives as long as the editor */
  if (!query || !canSave.value) return;
  const base: Widget = {
    ...props.widget,
    title: title.value.trim() || "Untitled",
    owner: target.value.owner,
    db: target.value.db,
  };
  delete base.refreshSeconds;
  if (refreshSeconds.value) base.refreshSeconds = refreshSeconds.value;
  emit("done", widgetFromQuery(base, query));
};
</script>

<template>
  <div
    class="widget-editor"
    role="dialog"
    aria-modal="true"
    :aria-label="header ?? 'Edit widget'"
    data-testid="widget-editor"
    @keydown.esc.self="emit('cancel')"
  >
    <!-- Esc in the query view closes its own hints and menus, not the editor. -->
    <header @keydown.esc="emit('cancel')">
      <h2>{{ header ?? "Edit widget" }}</h2>
      <div class="fields">
        <label class="field title">
          <span>Title</span>
          <input v-model="title" type="text" data-testid="widget-title" />
        </label>
        <label class="field">
          <span>Database</span>
          <select v-model="database" data-testid="widget-database">
            <option v-for="name in databaseOptions" :key="name" :value="name">
              {{ name }}
            </option>
          </select>
        </label>
        <label
          class="field small"
          :title="`0 = off, ${MIN_REFRESH_SECONDS} to ${MAX_REFRESH_SECONDS}`"
        >
          <span>Refresh (s)</span>
          <input
            v-model="refresh"
            type="number"
            min="0"
            :max="MAX_REFRESH_SECONDS"
            data-testid="widget-refresh-seconds"
            :aria-invalid="!!refreshError"
          />
        </label>
      </div>
      <div class="buttons">
        <button
          type="button"
          class="button button-transparent"
          data-testid="widget-editor-cancel"
          @click="emit('cancel')"
        >
          Cancel
        </button>
        <button
          type="button"
          class="button button-success"
          data-testid="widget-editor-done"
          :disabled="!canSave"
          @click="done"
        >
          Done
        </button>
      </div>
      <p class="hint" data-testid="widget-editor-hint">
        <template v-if="refreshError">{{ refreshError }}.</template>
        <template v-else-if="!target.db">Pick a database.</template>
        <template v-else-if="!hasQuery">
          Build a query in the exec tab and run it to preview.
        </template>
        <template v-else-if="unfinished">
          Finish or remove the rows that cannot run yet.
        </template>
        <template v-else>
          Exec and context rows, colours and the chosen view are saved with the
          widget.
        </template>
      </p>
    </header>
    <QueryView :query-id="queryId" :tabs="WIDGET_TABS" />
  </div>
</template>

<style lang="less" scoped>
.widget-editor {
  position: fixed;
  inset: 0;
  z-index: var(--z-index-dropdown);
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1rem 1.5rem;
  overflow: auto;
  background: var(--color-background);
}
header {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: end;
  gap: 0.5rem 1.5rem;
  h2 {
    margin: 0;
    align-self: center;
  }
}
.fields {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.85rem;
  color: var(--color-text-muted);
  input,
  select {
    min-width: 10rem;
  }
  &.title input {
    min-width: 16rem;
  }
  &.small input,
  &.small select {
    min-width: 5rem;
  }
  input[aria-invalid="true"] {
    outline: 1px solid var(--error-color);
  }
}
.buttons {
  display: flex;
  gap: 0.5rem;
}
.hint {
  grid-column: 1 / -1;
  margin: 0;
  font-size: 0.85rem;
  color: var(--color-text-muted);
}
</style>
