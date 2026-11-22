<script lang="ts" setup>
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { onBeforeRouteLeave, onBeforeRouteUpdate } from "vue-router";
import { useEventListener } from "@vueuse/core";
import { ChPlus, MdRefresh } from "@kalimahapps/vue-icons";
import SpinnerIcon from "@agdb-studio/design/src/components/icons/SpinnerIcon.vue";
import { useDbStore } from "@agdb-studio/db/src/composables/dbStore";
import {
  adminMode,
  canEdit,
  dashboardLabel,
  useDashboardStore,
  type RemovedWidget,
} from "../composables/dashboardStore";
import {
  NewerDashboardError,
  createWidget,
  orderedWidgets,
} from "../utils/dashboardDocument";
import type { Widget } from "../types";
import DashboardWidget from "./DashboardWidget.vue";
import WidgetEditor from "./WidgetEditor.vue";

const props = defineProps<{ owner: string; name: string }>();

const store = useDashboardStore();
const { current, dirty, saving, saveFailed } = store;
const { fetchDatabases } = useDbStore();

const loading = ref(true);
/** Why the dashboard could not be loaded, if it could not. */
const failed = ref<string>();
const refreshKey = ref(0);

/** The last removed widget, offered back for a while. */
const removed = ref<RemovedWidget>();
const UNDO_MS = 8000;
let undoTimer: ReturnType<typeof setTimeout> | undefined;
const clearUndo = () => {
  clearTimeout(undoTimer);
  removed.value = undefined;
};
onUnmounted(clearUndo);

let loadRun = 0;
/** The dashboard this view opened, closed again when the view goes away. */
let opened: typeof current.value;
const load = async () => {
  // Only the latest load (e.g. after quick name changes) updates the page.
  const run = ++loadRun;
  loading.value = true;
  failed.value = undefined;
  clearUndo();
  try {
    await store.loadDashboard({ owner: props.owner, name: props.name });
    if (run === loadRun) opened = current.value;
  } catch (error) {
    if (run !== loadRun) return;
    // The API error interceptor shows a notification for request errors.
    failed.value =
      error instanceof NewerDashboardError
        ? error.message
        : "The dashboard could not be loaded.";
  } finally {
    if (run === loadRun) loading.value = false;
  }
};

onMounted(load);
watch(() => [props.owner, props.name], load);

/**
 * Edits save at once; only those whose save failed would be lost. A running
 * save is waited for, so its failure is not missed after leaving.
 */
const confirmDiscard = async () => {
  await store.settleSaves();
  return (
    !saveFailed.value ||
    window.confirm("Some dashboard changes could not be saved. Discard them?")
  );
};

onBeforeRouteLeave(confirmDiscard);
// Closed on unmount, not when leaving: a navigation that ends on this same
// dashboard (e.g. the logo's start page redirecting to the home dashboard)
// keeps the view without loading it again.
onUnmounted(() => {
  if (opened && current.value === opened) store.closeDashboard();
});
// Switching to another dashboard reuses this component.
onBeforeRouteUpdate((to, from) =>
  to.params.owner === from.params.owner && to.params.name === from.params.name
    ? true
    : confirmDiscard(),
);
useEventListener(window, "beforeunload", (event: BeforeUnloadEvent) => {
  if (dirty.value) event.preventDefault();
});

const ensureDatabases = async () => {
  try {
    await fetchDatabases();
  } catch {
    // The editor still lets the widget keep its current database.
  }
};

/** The widget open in the editor; `adding` when it is not on the dashboard yet. */
const editor = ref<{ widget: Widget; adding: boolean }>();

const addWidget = async () => {
  await ensureDatabases();
  editor.value = { widget: createWidget({ owner: "", db: "" }), adding: true };
};

const editWidget = async (widget: Widget) => {
  await ensureDatabases();
  editor.value = { widget, adding: false };
};

const finishEditing = (widget: Widget, adding: boolean) => {
  if (adding) store.addWidget(widget);
  else store.updateWidget(widget);
  editor.value = undefined;
};

/** Read-only users see the widgets but cannot change them. */
const editable = computed(() => !!current.value && canEdit(current.value.role));

const placed = computed(() =>
  current.value ? orderedWidgets(current.value.document) : [],
);

const removeWidget = (id: string) => {
  clearUndo();
  removed.value = store.removeWidget(id);
  undoTimer = setTimeout(clearUndo, UNDO_MS);
};

const undoRemove = (widget: RemovedWidget) => {
  store.restoreWidget(widget);
  clearUndo();
};
</script>

<template>
  <div class="dashboard-view">
    <header class="toolbar">
      <h2>{{ dashboardLabel({ owner, name }) }}</h2>
      <span
        v-if="current && !editable"
        class="badge"
        title="You can view this dashboard but not change it"
        data-testid="dashboard-read-only"
      >
        Read only
      </span>
      <span
        v-if="current?.public"
        class="badge"
        title="All users can view this dashboard"
        data-testid="dashboard-public-badge"
      >
        Public
      </span>
      <span
        v-if="current && adminMode()"
        class="badge"
        title="Viewed as the server admin: widgets run their queries with admin access to every database"
        data-testid="dashboard-admin-badge"
      >
        Admin access
      </span>
      <span
        v-if="removed"
        class="status"
        role="status"
        data-testid="dashboard-removed"
      >
        Removed "{{ removed.widget.title || "Untitled" }}"
        <button
          type="button"
          class="link"
          data-testid="dashboard-undo"
          @click="undoRemove(removed)"
        >
          Undo
        </button>
      </span>
      <span
        v-if="saveFailed"
        class="status unsaved"
        role="alert"
        data-testid="dashboard-unsaved"
      >
        Changes not saved
        <button
          type="button"
          class="link"
          data-testid="dashboard-retry"
          :disabled="saving"
          @click="store.saveDashboard"
        >
          Retry
        </button>
      </span>
      <span
        v-else-if="saving"
        class="status"
        role="status"
        data-testid="dashboard-saving"
      >
        Saving…
      </span>
      <div class="buttons">
        <button
          type="button"
          class="button"
          title="Refresh all widgets"
          data-testid="dashboard-refresh"
          @click="refreshKey++"
        >
          <MdRefresh /> Refresh
        </button>
        <button
          v-if="editable || !current"
          type="button"
          class="button"
          data-testid="dashboard-add-widget"
          :disabled="!current"
          @click="addWidget"
        >
          <ChPlus /> Add widget
        </button>
      </div>
    </header>

    <SpinnerIcon v-if="loading" />
    <p v-else-if="failed" class="empty" data-testid="dashboard-failed">
      {{ failed }}
    </p>
    <p v-else-if="!placed.length" class="empty" data-testid="dashboard-empty">
      This dashboard has no widgets yet.
      <template v-if="editable"
        >Click "Add widget", or pin a query from the query page.</template
      >
    </p>
    <div v-else class="grid" data-testid="dashboard-grid">
      <DashboardWidget
        v-for="({ widget, layout }, index) of placed"
        :key="widget.id"
        :widget="widget"
        :size="layout"
        :first="index === 0"
        :last="index === placed.length - 1"
        :refresh-key="refreshKey"
        :read-only="!editable"
        @edit="editWidget(widget)"
        @remove="removeWidget(widget.id)"
        @move="(delta) => store.moveWidget(widget.id, delta)"
        @resize="(size) => store.resizeWidget(widget.id, size)"
        @update="store.updateWidget"
      />
    </div>
    <WidgetEditor
      v-if="editor"
      :key="editor.widget.id"
      :widget="editor.widget"
      :header="editor.adding ? 'Add widget' : 'Edit widget'"
      @done="finishEditing($event, editor.adding)"
      @cancel="editor = undefined"
    />
  </div>
</template>

<style lang="less" scoped>
.dashboard-view {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  width: 100%;
}
.toolbar {
  display: flex;
  align-items: center;
  gap: 1rem;
  h2 {
    margin: 0;
  }
  .buttons {
    margin-left: auto;
    display: flex;
    gap: 0.5rem;
  }
  .button {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
  }
}
.badge {
  color: var(--color-text-muted);
  font-size: 0.8rem;
  border: 1px solid var(--color-border);
  border-radius: 0.25rem;
  padding: 0 0.35rem;
}
.status {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.85rem;
  color: var(--color-text-muted);
  &.unsaved {
    color: var(--warning-color);
  }
}
.link {
  padding: 0;
  border: none;
  background: none;
  color: var(--color-text);
  font: inherit;
  text-decoration: underline;
  cursor: pointer;
  &:disabled {
    opacity: 0.5;
    cursor: default;
  }
}
.empty {
  color: var(--color-text-muted);
  text-align: center;
}
.grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  grid-auto-rows: 16rem;
  gap: 1rem;
}
@media (max-width: 900px) {
  .grid {
    grid-template-columns: minmax(0, 1fr);
    :deep(.dashboard-widget) {
      grid-column: auto !important;
    }
  }
}
</style>
