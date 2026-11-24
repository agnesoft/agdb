<script lang="ts" setup>
import type { DbElement, DbKeyValue, QueryResult } from "@agnesoft/agdb_api";
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
import ResultsChart from "./chart/ResultsChart.vue";
import ColorsPanel from "../graph/ColorsPanel.vue";
import { LuPalette } from "@kalimahapps/vue-icons";
import ColorLegend from "../graph/ColorLegend.vue";
import { useGraphPalette } from "../../composables/graphPalette";
import { hasColoring, type PaletteSlot } from "../../utils/graphColorRules";
import {
  elementLabel,
  provideGraphActions,
} from "../../composables/graphActions";
import { useQueryStore } from "../../composables/queryStore";
import {
  convertElementsToGraphData,
  dedupeResultElements,
} from "../../utils/graphDataUtils";
import {
  classifyResults,
  scalarLabel,
  summarizeResults,
  type ResultView,
} from "../../utils/resultShape";
import {
  resultKeys,
  suggestChart,
  type ChartConfig,
} from "../../utils/chartConfig";

const props = defineProps<{
  /**
   * Keep a chosen view, and pins of elements missing from the results, even
   * when the results change, e.g. on a dashboard re-running a saved query
   * whose results may briefly be empty.
   */
  keepView?: boolean;
  /** Little room (e.g. a dashboard widget): chart settings open in a popover. */
  compact?: boolean;
}>();

const queryId = inject<Ref<string | undefined>>("queryId");
const queryStore = useQueryStore();
const actions = provideGraphActions(queryId, {
  keepMissingPins: props.keepView,
});

const query = computed(() =>
  queryId?.value ? queryStore.getQuery(queryId.value)?.value : undefined,
);

const lastTab = computed(() => query.value?.lastExecutedTab);

const mainResults = computed<QueryResult[]>(() =>
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

// A view picked earlier (kept in the store, e.g. for a dashboard widget) wins.
const storedView = query.value?.view;
const view = ref<ResultView>(storedView ?? autoView.value);
const pinned = ref(storedView !== undefined);
let lastAutoView = autoView.value;

const storeView = (next: ResultView | undefined) => {
  if (queryId?.value) queryStore.setView(queryId.value, next);
};

// After each run pick the best view, unless the user chose one and the
// shape of the results did not change (or the view is kept).
watch(
  () => query.value?.lastRun,
  () => {
    const auto = autoView.value;
    const shapeChanged = !props.keepView && auto !== lastAutoView;
    if (!pinned.value || shapeChanged) {
      view.value = auto;
      if (pinned.value) storeView(undefined);
      pinned.value = false;
    }
    lastAutoView = auto;
  },
);

const selectView = (next: ResultView) => {
  // A chart starts from one that fits the results, and is kept with them.
  // Before any results it stays unset (the chart view suggests one as they
  // come); one set up for other results (keys they lack) starts over, but
  // not for results without elements, which say what is missing instead.
  const saved = query.value?.chart;
  if (
    next === "chart" &&
    lastTab.value &&
    (!saved ||
      (mainResults.value.some((r) => r.elements.length) && !chartFits(saved)))
  ) {
    setChart(suggestChart(mainResults.value));
  }
  view.value = next;
  pinned.value = true;
  storeView(next);
};

const viewOptions = computed(() => {
  const options: { key: ResultView; label: string }[] = [
    { key: "graph", label: "Graph" },
    { key: "table", label: "Table" },
    { key: "chart", label: "Chart" },
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

/** Distinct elements, shown when queries of the batch returned some of the same ones. */
const uniqueCount = computed(() => {
  const total = mainResults.value.reduce((n, r) => n + r.elements.length, 0);
  const unique = dedupeResultElements(mainResults.value).length;
  return unique < total ? unique : undefined;
});

const extraCounts = computed(() => {
  const count = (results?: QueryResult[]) =>
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

/** The chart uses only keys the results have. */
const chartFits = (chart: ChartConfig): boolean => {
  const keys = new Set(resultKeys(mainResults.value).map((k) => k.key));
  const used = [
    ...(chart.groupBy.by === "property" ? [chart.groupBy.key] : []),
    ...(chart.measure.aggregate === "count" ? [] : [chart.measure.key]),
  ];
  return used.every((key) => keys.has(key));
};

const setChart = (config: ChartConfig) => {
  if (queryId?.value) queryStore.setChart(queryId.value, config);
};

/** The queries of the batch, as the chart labels its groups by query. */
const queryLabels = computed(() =>
  actions.colorQueries.value.map((q) => `#${q.index + 1} ${q.label}`),
);

const lastRunLabel = computed(() =>
  query.value?.lastRun ? query.value.lastRun.toLocaleTimeString() : "",
);

const menuActions = computed(() =>
  actions.menu.value ? actions.getActions(actions.menu.value.element) : [],
);

const openTableMenu = (element: DbElement, x: number, y: number) => {
  const known = actions.elementsById.value.get(element.id);
  const converted = convertElementsToGraphData([element], "query");
  const graphElement = known ?? converted.nodes[0] ?? converted.links[0];
  /* v8 ignore else -- @preserve: id 0 rows have no menu */
  if (graphElement) actions.openMenu(graphElement, x, y);
};

const palette = useGraphPalette();
const colorsOpen = ref(false);
const colored = computed(() => hasColoring(query.value?.colors));
/** The colours in use, shown on the toolbar button. */
const usedColors = computed(() => [
  ...new Set(actions.colorLegend.value.map((entry) => entry.slot)),
]);
const menuColor = computed(() => {
  const element = actions.menu.value?.element;
  return element ? query.value?.colors?.elements[element.id] : undefined;
});
const setMenuColor = (slot: PaletteSlot | undefined) => {
  const element = actions.menu.value?.element;
  /* v8 ignore else -- @preserve: the menu is open while picking */
  if (element) actions.setElementColor(element, slot);
};

const saving = ref(false);
const saveValues = async (values: DbKeyValue[], original: DbKeyValue[]) => {
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
          <span
            v-if="uniqueCount !== undefined"
            class="chip chip-muted"
            data-testid="results-unique-chip"
            title="Elements returned by more than one query are shown once in the graph and table"
          >
            unique: {{ uniqueCount }}
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
      <div class="toolbar-actions">
        <div v-if="view !== 'chart'" class="colors-control">
          <button
            type="button"
            class="button colors-button"
            :class="{ colored }"
            :aria-expanded="colorsOpen"
            data-testid="colors-toggle"
            title="Colour nodes and edges by query, by property or one by one"
            @click="colorsOpen = !colorsOpen"
          >
            <LuPalette class="colors-icon" />
            Colours
            <span
              v-if="usedColors.length"
              class="used-colors"
              aria-hidden="true"
            >
              <span
                v-for="slot in usedColors"
                :key="slot"
                :style="{ backgroundColor: palette[slot] }"
              ></span>
            </span>
          </button>
          <ColorsPanel
            v-if="colorsOpen"
            class="colors-popover"
            :palette="palette"
            :queries="actions.colorQueries.value"
            :rules="actions.colorRules.value"
            :keys="actions.valueIndex.value"
            :elements="actions.colorElements.value"
            @row-color="actions.setRowColor"
            @each-query="actions.colorEachQuery()"
            @update:rules="actions.setColorRules"
            @element-color="actions.setElementColorById"
            @clear-all="actions.clearColors()"
            @close="colorsOpen = false"
          />
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
    </div>

    <div class="results-body">
      <QueryGraph v-if="view === 'graph'" />
      <ResultsTable
        v-else-if="view === 'table'"
        :results="mainResults"
        :alias-of="actions.aliasOf"
        @menu="openTableMenu"
      />
      <ResultsChart
        v-else-if="view === 'chart'"
        :results="mainResults"
        :config="query?.chart"
        :query-labels="queryLabels"
        :ran="!!lastTab"
        :compact="compact"
        @update:config="setChart"
      />
      <ResultsJson v-else-if="view === 'json'" :results="mainResults" />
      <ResultScalar v-else :label="scalar.label" :value="scalar.value" />
      <!-- After the view chain: between its v-if and v-else it would break it. -->
      <ColorLegend
        v-if="view === 'graph' && actions.colorLegend.value.length"
        :entries="actions.colorLegend.value"
        :palette="palette"
        @edit="colorsOpen = true"
      />

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
      :title="elementLabel(actions.describeElement(actions.menu.value.element))"
      :actions="menuActions"
      :palette="palette"
      :color="menuColor"
      @color="setMenuColor"
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
.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}
.colors-control {
  position: relative;
}
.colors-button {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  &.colored {
    border-color: var(--color-text-muted);
  }
}
.colors-icon {
  font-size: 1rem;
}
.used-colors {
  display: inline-flex;
  gap: 2px;
  span {
    width: 0.55rem;
    height: 0.55rem;
    border-radius: 50%;
  }
}
.colors-popover {
  position: absolute;
  top: calc(100% + 0.25rem);
  right: 0;
  z-index: var(--z-index-dropdown);
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
