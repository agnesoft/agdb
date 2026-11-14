<script lang="ts" setup>
import type { Components } from "@agnesoft/agdb_api";
import {
  computed,
  inject,
  onMounted,
  onUnmounted,
  ref,
  watch,
  type Ref,
} from "vue";
import QueryGraph from "../graph/QueryGraph.vue";
import GraphContextMenu from "../graph/GraphContextMenu.vue";
import ElementDetailsPanel from "../graph/ElementDetailsPanel.vue";
import ResultsTable from "./ResultsTable.vue";
import ResultsJson from "./ResultsJson.vue";
import ResultScalar from "./ResultScalar.vue";
import {
  elementLabel,
  provideGraphActions,
} from "../../composables/graphActions";
import { useQueryStore } from "../../composables/queryStore";
import { convertElementsToGraphData } from "../../utils/graphDataUtils";
import {
  classifyResults,
  scalarLabel,
  summarizeResults,
  type ResultView,
} from "../../utils/resultShape";

const queryId = inject<Ref<string>>("queryId");
const queryStore = useQueryStore();
const actions = provideGraphActions(queryId);

const query = computed(() =>
  queryId?.value ? queryStore.getQuery(queryId.value)?.value : undefined,
);

const lastTab = computed(() => query.value?.lastExecutedTab);

const mainResults = computed<Components.Schemas.QueryResult[]>(() =>
  lastTab.value ? (query.value?.results[lastTab.value] ?? []) : [],
);

// The steps that produced the results, not the builder's current (maybe unrun) rows.
const stepTypes = computed(() =>
  lastTab.value ? (query.value?.executedStepTypes?.[lastTab.value] ?? []) : [],
);

/** Before the first run the (empty) graph is shown, as before. */
const autoView = computed<Exclude<ResultView, "json">>(() =>
  lastTab.value ? classifyResults(mainResults.value, stepTypes.value) : "graph",
);

const view = ref<ResultView>(autoView.value);
const pinned = ref(false);
let lastAutoView = autoView.value;

// After each run pick the best view, unless the user chose one and the
// shape of the results did not change.
watch(
  () => query.value?.lastRun,
  () => {
    const auto = autoView.value;
    if (!pinned.value || auto !== lastAutoView) {
      view.value = auto;
      pinned.value = false;
    }
    lastAutoView = auto;
  },
);

const selectView = (next: ResultView) => {
  view.value = next;
  pinned.value = true;
};

const viewOptions = computed(() => {
  const options: { key: ResultView; label: string }[] = [
    { key: "graph", label: "Graph" },
    { key: "table", label: "Table" },
    { key: "json", label: "JSON" },
  ];
  if (autoView.value === "scalar") {
    options.unshift({ key: "scalar", label: "Summary" });
  }
  return options;
});

const chips = computed(() => {
  const summaries = summarizeResults(mainResults.value);
  return summaries.map((summary) => {
    const parts = [`result: ${summary.result.toLocaleString()}`];
    if (summary.nodes) parts.push(`${summary.nodes} nodes`);
    if (summary.edges) parts.push(`${summary.edges} edges`);
    if (summary.other) parts.push(`${summary.other} other`);
    if (summaries.length > 1) parts.unshift(`#${summary.index + 1}`);
    return { index: summary.index, label: parts.join(" · ") };
  });
});

const extraCounts = computed(() => {
  const count = (results?: Components.Schemas.QueryResult[]) =>
    (results ?? []).reduce((sum, r) => sum + r.elements.length, 0);
  return {
    context: count(query.value?.results.context),
    expanded: count(query.value?.results.expanded),
  };
});

const scalar = computed(() => ({
  label: scalarLabel(stepTypes.value, lastTab.value === "exec_mut"),
  value: mainResults.value.reduce((sum, r) => sum + r.result, 0),
}));

const lastRunLabel = computed(() =>
  query.value?.lastRun ? query.value.lastRun.toLocaleTimeString() : "",
);

const menuActions = computed(() =>
  actions.menu.value ? actions.getActions(actions.menu.value.element) : [],
);

const openTableMenu = (
  element: Components.Schemas.DbElement,
  x: number,
  y: number,
) => {
  const known = actions.elementsById.value.get(element.id);
  const converted = convertElementsToGraphData([element], "query");
  const graphElement = known ?? converted.nodes[0] ?? converted.links[0];
  /* v8 ignore else -- @preserve: id 0 rows have no menu */
  if (graphElement) actions.openMenu(graphElement, x, y);
};

const saving = ref(false);
const saveValues = async (
  values: Components.Schemas.DbKeyValue[],
  original: Components.Schemas.DbKeyValue[],
) => {
  const element = actions.selectedElement.value;
  /* v8 ignore next -- @preserve */
  if (!element) return;
  saving.value = true;
  try {
    await actions.saveValues(element, values, original);
  } finally {
    saving.value = false;
  }
};

const onKeydown = (event: KeyboardEvent) => {
  if (
    event.key === "Escape" &&
    actions.pendingEdgeSourceId.value !== undefined
  ) {
    actions.cancelAddEdge();
  }
};

onMounted(() => document.addEventListener("keydown", onKeydown));
onUnmounted(() => document.removeEventListener("keydown", onKeydown));
</script>

<template>
  <section class="query-results" data-testid="query-results">
    <div class="results-toolbar">
      <div class="summary" data-testid="results-summary">
        <template v-if="lastTab">
          <span
            v-for="chip in chips"
            :key="chip.index"
            class="chip"
            data-testid="results-chip"
          >
            {{ chip.label }}
          </span>
          <span v-if="extraCounts.context" class="chip chip-muted">
            context: {{ extraCounts.context }}
          </span>
          <span v-if="extraCounts.expanded" class="chip chip-muted">
            expanded: {{ extraCounts.expanded }}
          </span>
          <span v-if="lastRunLabel" class="last-run">{{ lastRunLabel }}</span>
        </template>
        <span v-else class="last-run">Run a query to see results.</span>
      </div>
      <div class="view-toggle" role="group" aria-label="Result view">
        <button
          v-for="option in viewOptions"
          :key="option.key"
          type="button"
          class="button button-tab"
          :class="{ active: view === option.key }"
          :aria-pressed="view === option.key"
          :data-testid="`view-${option.key}`"
          @click="selectView(option.key)"
        >
          {{ option.label }}
        </button>
      </div>
    </div>

    <div class="results-body">
      <QueryGraph v-if="view === 'graph'" />
      <ResultsTable
        v-else-if="view === 'table'"
        :results="mainResults"
        @menu="openTableMenu"
      />
      <ResultsJson v-else-if="view === 'json'" :results="mainResults" />
      <ResultScalar v-else :label="scalar.label" :value="scalar.value" />

      <div
        v-if="actions.pendingEdgeSourceId.value !== undefined"
        class="add-edge-banner"
        data-testid="add-edge-banner"
      >
        Adding edge from node {{ actions.pendingEdgeSourceId.value }} — select
        the target node in the graph (Esc to cancel)
        <button
          type="button"
          class="button"
          data-testid="add-edge-cancel"
          @click="actions.cancelAddEdge()"
        >
          Cancel
        </button>
      </div>

      <ElementDetailsPanel
        v-if="actions.selectedElement.value"
        :element="actions.selectedElement.value"
        :mode="actions.panelMode.value"
        :saving="saving"
        @close="actions.closePanel()"
        @edit="actions.editElement(actions.selectedElement.value)"
        @cancel="actions.cancelEdit()"
        @dirty="actions.setUnsavedEdits"
        @save="saveValues"
        @retry="actions.loadValues(actions.selectedElement.value.id)"
      />
    </div>

    <GraphContextMenu
      v-if="actions.menu.value"
      :x="actions.menu.value.x"
      :y="actions.menu.value.y"
      :title="elementLabel(actions.menu.value.element)"
      :actions="menuActions"
      @close="actions.closeMenu()"
    />
  </section>
</template>

<style lang="less" scoped>
.query-results {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  grid-template-rows: max-content 1fr;
  gap: 0.5rem;
  min-width: 0;
  min-height: 0;
}
.results-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 0.5rem;
}
.summary {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.4rem;
  font-size: 0.85rem;
}
.chip {
  padding: 0.15rem 0.6rem;
  border-radius: 1rem;
  border: 1px solid var(--color-border);
  background-color: var(--color-background-mute);
}
.chip-muted,
.last-run {
  color: var(--color-text-muted);
}
.view-toggle {
  display: flex;
  gap: 0.25rem;
}
.results-body {
  position: relative;
  min-width: 0;
  min-height: 0;
}
.add-edge-banner {
  position: absolute;
  top: 0.5rem;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.4rem 0.75rem;
  border-radius: 0.5rem;
  border: 1px solid var(--graph-selected-color);
  background-color: var(--color-background-mute);
  z-index: 2;
}
</style>
