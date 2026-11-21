<script lang="ts" setup>
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { watchDebounced } from "@vueuse/core";
import QueryView from "@agdb-studio/query/src/components/view/QueryView.vue";
import { provideDbTarget } from "@agdb-studio/query/src/composables/dbExec";
import { useQueryStore } from "@agdb-studio/query/src/composables/queryStore";
import { useDbStore } from "@agdb-studio/db/src/composables/dbStore";
import { WIDGET_TABS, type Widget } from "../types";
import {
  REFRESH_CHOICES,
  formatRefresh,
  isRefreshInterval,
} from "../utils/refresh";
import {
  hasUnfinishedRow,
  seedQuery,
  widgetFromQuery,
} from "../utils/widgetRows";
import { errorMessage, runEntryRows } from "../composables/widgetQuery";

const props = defineProps<{
  widget: Widget;
  header?: string;
}>();

const emit = defineEmits<{
  done: [widget: Widget];
  cancel: [];
}>();

/** Wait after the last builder edit before the preview runs again. */
const PREVIEW_DEBOUNCE_MS = 600;

const { databases } = useDbStore();
const queryStore = useQueryStore();

const title = ref(props.widget.title);
const database = ref(
  props.widget.owner
    ? `${props.widget.owner}/${props.widget.db}`
    : (databases.value.map((d) => `${d.owner}/${d.db}`)[0] ?? ""),
);

const savedRefresh = props.widget.refreshSeconds ?? 0;
const refresh = ref(isRefreshInterval(savedRefresh) ? savedRefresh : 0);
// A saved interval that is not among the choices stays selectable.
const refreshOptions = computed(() =>
  [
    ...new Set<number>([
      ...REFRESH_CHOICES,
      ...(refresh.value ? [refresh.value] : []),
    ]),
  ]
    .sort((a, b) => a - b)
    .map((seconds) => ({ seconds, label: formatRefresh(seconds) })),
);

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

const entry = computed(() => queryStore.getQuery(queryId)?.value);

const hasQuery = computed(
  () => !!entry.value?.rows.exec.some((row) => row.steps.length > 0),
);
const unfinished = computed(
  () => !!entry.value && hasUnfinishedRow(entry.value),
);
const canRun = computed(
  () => hasQuery.value && !unfinished.value && !!target.value.db,
);
const canSave = canRun;

/** Why the last preview failed, if it did. */
const previewError = ref<string>();
const previewing = ref(false);
let previewRun = 0;

/** Shows the widget's results as they would be on the dashboard. */
const preview = async () => {
  if (!canRun.value) return;
  const id = ++previewRun;
  previewing.value = true;
  try {
    // Previews run while typing: errors show in the hint, not as notifications.
    await runEntryRows(queryId, target.value, () => id === previewRun, {
      quiet: true,
    });
    if (id === previewRun) previewError.value = undefined;
  } catch (error) {
    if (id === previewRun) previewError.value = errorMessage(error);
  } finally {
    if (id === previewRun) previewing.value = false;
  }
};

onMounted(preview);
// A run from the query view's button wins over a preview still running.
watch(
  () => entry.value?.isRunning,
  (running) => {
    if (!running) return;
    previewRun++;
    previewing.value = false;
  },
);
// Results of another database must not stay when the preview cannot run.
watch(
  () => database.value,
  () => {
    if (!canRun.value) queryStore.clearQueryResults(queryId);
    void preview();
  },
);
// What the preview depends on in the builder: the rows' steps and arguments.
const rowsKey = computed(() =>
  JSON.stringify(
    WIDGET_TABS.map((tab) =>
      entry.value?.rows[tab].map((row) =>
        row.steps.map(({ type, args }) => ({ type, args })),
      ),
    ),
  ),
);
watchDebounced(rowsKey, preview, { debounce: PREVIEW_DEBOUNCE_MS });

/** What stands between the user and a working widget, if anything. */
const hint = computed<{ text: string; error?: boolean } | undefined>(() => {
  if (!target.value.db) return { text: "Pick a database." };
  if (!hasQuery.value) {
    return {
      text: "Build a query in the exec tab; the preview updates as you edit.",
    };
  }
  if (unfinished.value) {
    return { text: "Finish or remove the rows that cannot run yet." };
  }
  if (previewError.value) {
    return { text: `Preview failed: ${previewError.value}`, error: true };
  }
  return undefined;
});

const done = () => {
  const query = entry.value;
  /* v8 ignore next -- @preserve: the entry lives as long as the editor */
  if (!query || !canSave.value) return;
  const base: Widget = {
    ...props.widget,
    title: title.value.trim() || "Untitled",
    owner: target.value.owner,
    db: target.value.db,
  };
  delete base.refreshSeconds;
  if (refresh.value) base.refreshSeconds = refresh.value;
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
      <div class="top">
        <h2>{{ header ?? "Edit widget" }}</h2>
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
            title="Rows, colours, the chosen view and pinned details are saved with the widget"
            :disabled="!canSave"
            @click="done"
          >
            Save
          </button>
        </div>
      </div>
      <div class="fields">
        <label class="field title">
          <span>Title</span>
          <input
            v-model="title"
            type="text"
            placeholder="Untitled"
            data-testid="widget-title"
          />
        </label>
        <label class="field">
          <span>Database</span>
          <select v-model="database" data-testid="widget-database">
            <option v-for="name in databaseOptions" :key="name" :value="name">
              {{ name }}
            </option>
          </select>
        </label>
        <label class="field">
          <span>Auto refresh</span>
          <select v-model.number="refresh" data-testid="widget-refresh-seconds">
            <option
              v-for="option in refreshOptions"
              :key="option.seconds"
              :value="option.seconds"
            >
              {{ option.label }}
            </option>
          </select>
        </label>
        <p
          class="hint"
          :class="{ error: hint?.error }"
          role="status"
          data-testid="widget-editor-hint"
        >
          <template v-if="hint">{{ hint.text }}</template>
          <template v-else-if="previewing">Updating preview…</template>
        </p>
      </div>
    </header>
    <!-- Like on the dashboard: pins of elements missing from the results stay. -->
    <QueryView :query-id="queryId" :tabs="WIDGET_TABS" keep-view />
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
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding-bottom: 1rem;
  border-bottom: 1px solid var(--color-border);
}
.top {
  display: flex;
  align-items: center;
  gap: 1rem;
  h2 {
    margin: 0;
  }
}
.buttons {
  margin-left: auto;
  display: flex;
  gap: 0.5rem;
}
.fields {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-end;
  gap: 0.75rem 1rem;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.8rem;
  color: var(--color-text-muted);
  input,
  select {
    height: 2rem;
    min-width: 10rem;
    padding: 0 0.5rem;
    border: 1px solid var(--color-border);
    border-radius: 0.25rem;
    background: var(--color-background-soft);
    color: var(--color-text);
    font-size: 0.9rem;
    &:hover {
      border-color: var(--color-border-hover);
    }
    &:focus {
      outline: none;
      border-color: var(--color-border-hover);
    }
  }
  &.title {
    flex: 0 1 22rem;
    input {
      width: 100%;
    }
  }
}
.hint {
  flex: 1 1 12rem;
  // Room for one line, so the page does not move when it appears.
  min-height: 2rem;
  margin: 0;
  display: flex;
  align-items: center;
  font-size: 0.85rem;
  color: var(--color-text-muted);
  &.error {
    color: var(--error-color);
  }
}
</style>
