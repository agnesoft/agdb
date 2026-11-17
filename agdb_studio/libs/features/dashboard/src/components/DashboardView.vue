<script lang="ts" setup>
import { computed, onMounted, ref, watch } from "vue";
import { onBeforeRouteLeave, onBeforeRouteUpdate } from "vue-router";
import { useEventListener } from "@vueuse/core";
import { ChPlus, MdRefresh } from "@kalimahapps/vue-icons";
import SpinnerIcon from "@agdb-studio/design/src/components/icons/SpinnerIcon.vue";
import { useDbStore } from "@agdb-studio/db/src/composables/dbStore";
import { useDashboardStore } from "../composables/dashboardStore";
import {
  NewerDashboardError,
  createWidget,
  orderedWidgets,
} from "../utils/dashboardDocument";
import type { Widget } from "../types";
import DashboardWidget from "./DashboardWidget.vue";
import WidgetEditor from "./WidgetEditor.vue";

const props = defineProps<{ name: string }>();

const store = useDashboardStore();
const { current, dirty } = store;
const { fetchDatabases } = useDbStore();

const loading = ref(true);
/** Why the dashboard could not be loaded, if it could not. */
const failed = ref<string>();
const editing = ref(false);
const refreshKey = ref(0);

let loadRun = 0;
const load = async () => {
  // Only the latest load (e.g. after quick name changes) updates the page.
  const run = ++loadRun;
  loading.value = true;
  failed.value = undefined;
  editing.value = false;
  try {
    await store.loadDashboard(props.name);
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
watch(() => props.name, load);

const confirmDiscard = () =>
  !dirty.value || window.confirm("Discard unsaved dashboard changes?");

onBeforeRouteLeave(() => {
  if (!confirmDiscard()) return false;
  store.closeDashboard();
  return true;
});
// Switching to another dashboard reuses this component.
onBeforeRouteUpdate((to, from) =>
  to.params.name === from.params.name ? true : confirmDiscard(),
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

const placed = computed(() =>
  current.value ? orderedWidgets(current.value.document) : [],
);

const save = async () => {
  try {
    await store.saveDashboard();
    editing.value = false;
  } catch {
    // The API error interceptor shows a notification; edits are kept.
  }
};

const cancel = async () => {
  if (!confirmDiscard()) return;
  editing.value = false;
  await load();
};
</script>

<template>
  <div class="dashboard-view">
    <header class="toolbar">
      <h2>{{ name }}</h2>
      <span v-if="dirty" class="unsaved" data-testid="dashboard-unsaved">
        Unsaved changes
      </span>
      <div class="buttons">
        <template v-if="editing">
          <button
            type="button"
            class="button"
            data-testid="dashboard-add-widget"
            @click="addWidget"
          >
            <ChPlus /> Add widget
          </button>
          <button
            type="button"
            class="button"
            data-testid="dashboard-cancel"
            @click="cancel"
          >
            Cancel
          </button>
          <button
            type="button"
            class="button button-success"
            data-testid="dashboard-save"
            :disabled="!dirty"
            @click="save"
          >
            Save
          </button>
        </template>
        <template v-else>
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
            type="button"
            class="button"
            data-testid="dashboard-edit"
            :disabled="!current"
            @click="editing = true"
          >
            Edit
          </button>
        </template>
      </div>
    </header>

    <SpinnerIcon v-if="loading" />
    <p v-else-if="failed" class="empty" data-testid="dashboard-failed">
      {{ failed }}
    </p>
    <p v-else-if="!placed.length" class="empty" data-testid="dashboard-empty">
      This dashboard has no widgets yet.
      <template v-if="!editing"
        >Click "Edit" to add one, or pin a query from the query page.</template
      >
    </p>
    <div v-else class="grid" data-testid="dashboard-grid">
      <DashboardWidget
        v-for="({ widget, layout }, index) of placed"
        :key="widget.id"
        :widget="widget"
        :size="layout"
        :editing="editing"
        :first="index === 0"
        :last="index === placed.length - 1"
        :refresh-key="refreshKey"
        @edit="editWidget(widget)"
        @remove="store.removeWidget(widget.id)"
        @move="(delta) => store.moveWidget(widget.id, delta)"
        @resize="(size) => store.resizeWidget(widget.id, size)"
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
.unsaved {
  color: var(--warning-color);
  font-size: 0.85rem;
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
