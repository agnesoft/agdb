<script lang="ts" setup>
import type { QueryResult } from "@agnesoft/agdb_api";
import { computed, ref, useTemplateRef } from "vue";
import { onClickOutside, useElementSize } from "@vueuse/core";
import { LuSlidersHorizontal } from "@kalimahapps/vue-icons";
import {
  chartLabel,
  isSplit,
  resultKeys,
  showableChart,
  suggestChart,
  type ChartConfig,
} from "../../../utils/chartConfig";
import { computeChart, computeSeries } from "../../../utils/chartData";
import {
  OTHER_COLOR,
  formatValue,
  paletteColor,
} from "../../../utils/chartScale";
import ChartSettings from "./ChartSettings.vue";
import ChartBar from "./ChartBar.vue";
import ChartLine, { type LineSeries } from "./ChartLine.vue";
import ChartDonut from "./ChartDonut.vue";
import ChartStat from "./ChartStat.vue";

const props = defineProps<{
  results: QueryResult[];
  /** Unset shows a chart suggested from the results. */
  config?: ChartConfig;
  /** Labels of the queries of the batch, for charts by query. */
  queryLabels?: string[];
  /** The query has run (else there is nothing to chart yet). */
  ran?: boolean;
  /** Settings in a popover instead of a strip, e.g. on a dashboard widget. */
  compact?: boolean;
}>();

const emit = defineEmits<{
  "update:config": [config: ChartConfig];
}>();

/** The chart as stored (or suggested), and as it can be shown. */
const saved = computed(() => props.config ?? suggestChart(props.results));
const chart = computed(() => showableChart(saved.value));
const keys = computed(() => resultKeys(props.results));
const data = computed(() =>
  computeChart(props.results, chart.value, props.queryLabels),
);
const label = computed(() => chartLabel(chart.value));

/** A line per query of the batch, along the property grouped by. */
const series = computed(() => {
  const { groupBy, measure } = chart.value;
  if (!isSplit(chart.value) || groupBy.by !== "property") return undefined;
  return computeSeries(props.results, groupBy.key, measure, props.queryLabels);
});

/** Donut slices need a positive share of the whole. */
const shown = computed(() =>
  chart.value.kind === "donut"
    ? data.value.points.filter((p) => p.value > 0)
    : data.value.points,
);

/** What the line chart draws: the series, or the one line of the points. */
const lines = computed<{
  xs: { key: string; label: string }[];
  series: LineSeries[];
}>(() => {
  if (series.value) {
    return {
      xs: series.value.xs,
      series: series.value.series.map((s) => ({
        key: s.key,
        label: s.label,
        short: s.short,
        color: s.slot === undefined ? OTHER_COLOR : paletteColor(s.slot),
        values: s.values,
      })),
    };
  }
  return {
    xs: shown.value,
    series: [
      {
        key: "value",
        label: label.value,
        color: paletteColor(0),
        values: shown.value.map((p) => p.value),
      },
    ],
  };
});

const hasData = computed(() =>
  series.value ? series.value.xs.length > 0 : shown.value.length > 0,
);

/** Why there is no chart, with what to do about it. */
const empty = computed<string | undefined>(() => {
  if (!props.ran) return "Nothing to chart yet — run the query.";
  if (hasData.value) return undefined;
  const { groupBy, measure } = chart.value;
  const has = (key: string) => keys.value.some((k) => k.key === key);
  if (!props.results.length) return "The query returned no results.";
  if (
    chart.value.kind !== "stat" &&
    groupBy.by === "property" &&
    !has(groupBy.key)
  ) {
    return `No element has "${groupBy.key}" — pick another property to group by.`;
  }
  if (chart.value.kind === "donut" && data.value.points.length) {
    return "A donut needs positive values — try a bar chart.";
  }
  if (measure.aggregate !== "count") {
    return `No element has a number in "${measure.key}" — pick another value to show.`;
  }
  /* v8 ignore next -- @preserve: counts give a point for every key or query */
  return "A donut needs positive values — try a bar chart.";
});

/** What was left out of the chart, so a smaller number is not a surprise. */
const footnote = computed(() => {
  if (empty.value) return undefined;
  const notes: string[] = [];
  const { skipped, folded } = series.value ?? data.value;
  const { groupBy, measure } = chart.value;
  if (skipped) {
    const keysUsed = [
      ...(chart.value.kind !== "stat" && groupBy.by === "property"
        ? [groupBy.key]
        : []),
      ...(measure.aggregate !== "count" ? [measure.key] : []),
    ];
    notes.push(
      `${skipped} ${skipped === 1 ? "element" : "elements"} without ${keysUsed
        .map((k) => `"${k}"`)
        .join(" or ")} left out`,
    );
  }
  if (folded && series.value) {
    notes.push(`${folded} more queries in "Other"`);
  } else if (folded) {
    const what = groupBy.by === "query" ? "queries" : "groups";
    notes.push(`${folded} smaller ${what} in "Other"`);
  }
  const dropped = data.value.points.length - shown.value.length;
  if (dropped) {
    const groups = dropped === 1 ? "group" : "groups";
    notes.push(`${dropped} ${groups} of zero or less not shown`);
  }
  return notes.join(" · ") || undefined;
});

const area = useTemplateRef<HTMLElement>("area");
const { width, height } = useElementSize(area);
const size = computed(() => ({
  width: Math.floor(width.value),
  height: Math.floor(height.value),
}));

const settingsOpen = ref(false);
const popover = useTemplateRef<HTMLElement>("popover");
onClickOutside(popover, () => (settingsOpen.value = false));

const update = (next: ChartConfig) => emit("update:config", next);
</script>

<template>
  <div class="results-chart" :class="{ compact }" data-testid="results-chart">
    <ChartSettings
      v-if="!compact"
      class="strip"
      :config="chart"
      :saved="saved"
      :keys="keys"
      :queries="results.length"
      @update:config="update"
    />
    <div class="head">
      <span class="title" data-testid="chart-title">{{ label }}</span>
      <div
        v-if="compact"
        ref="popover"
        class="settings-control"
        @keydown.esc.stop="settingsOpen = false"
      >
        <button
          type="button"
          class="settings-toggle"
          title="Chart settings"
          aria-label="Chart settings"
          :aria-expanded="settingsOpen"
          data-testid="chart-settings-toggle"
          @click="settingsOpen = !settingsOpen"
        >
          <LuSlidersHorizontal />
        </button>
        <ChartSettings
          v-if="settingsOpen"
          class="popover"
          :config="chart"
          :saved="saved"
          :keys="keys"
          :queries="results.length"
          @update:config="update"
        />
      </div>
    </div>
    <div ref="area" class="area">
      <p v-if="empty" class="empty" data-testid="chart-empty">{{ empty }}</p>
      <template v-else-if="size.width > 0 && size.height > 0">
        <ChartStat
          v-if="chart.kind === 'stat'"
          :value="shown[0]?.value ?? 0"
          :label="label"
        />
        <ChartBar
          v-else-if="chart.kind === 'bar'"
          :points="shown"
          :width="size.width"
          :height="size.height"
          :label="label"
        />
        <ChartLine
          v-else-if="chart.kind === 'line'"
          :xs="lines.xs"
          :series="lines.series"
          :axis="series?.axis ?? data.axis"
          :width="size.width"
          :height="size.height"
          :label="label"
        />
        <ChartDonut
          v-else
          :points="shown"
          :width="size.width"
          :height="size.height"
          :label="label"
        />
      </template>
    </div>
    <p v-if="footnote" class="footnote" data-testid="chart-footnote">
      {{ footnote }}
    </p>
    <!-- The chart's numbers for screen readers. -->
    <table v-if="!empty" class="visually-hidden" data-testid="chart-data-table">
      <caption>
        {{
          label
        }}
      </caption>
      <template v-if="series">
        <thead>
          <tr>
            <td />
            <th v-for="s in series.series" :key="s.key" scope="col">
              {{ s.label }}
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(x, i) in series.xs" :key="x.key">
            <th scope="row">{{ x.label }}</th>
            <td v-for="s in series.series" :key="s.key">
              {{ s.values[i] === undefined ? "–" : formatValue(s.values[i]) }}
            </td>
          </tr>
        </tbody>
      </template>
      <tbody v-else>
        <tr v-for="point in shown" :key="point.key">
          <th scope="row">{{ point.label }}</th>
          <td>{{ formatValue(point.value) }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style lang="less" scoped>
.results-chart {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  height: 100%;
  min-height: 20rem;
  padding: 0.75rem;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  background-color: var(--graph-background);
  &.compact {
    min-height: 0;
    padding: 0.5rem;
  }
}
.strip {
  padding-bottom: 0.5rem;
  border-bottom: 1px solid var(--color-border);
}
.head {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-height: 1.5rem;
}
.title {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--color-text-muted);
  font-size: 0.8rem;
}
// The popover is placed within the chart (not under the button), so it
// stays inside a small widget and scrolls there.
.settings-control {
  margin-left: auto;
}
.settings-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.5rem;
  height: 1.5rem;
  padding: 0;
  border: none;
  border-radius: 0.25rem;
  background: transparent;
  color: var(--color-text-muted);
  cursor: pointer;
  &:hover,
  &[aria-expanded="true"] {
    background: var(--color-background-mute);
    color: var(--color-text);
  }
}
.popover {
  position: absolute;
  top: 2.25rem;
  right: 0.5rem;
  z-index: var(--z-index-dropdown);
  flex-direction: column;
  align-items: stretch;
  width: max-content;
  max-width: min(20rem, calc(100% - 1rem));
  max-height: calc(100% - 2.75rem);
  overflow: auto;
  padding: 0.6rem;
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  background: var(--color-background);
  box-shadow: 0 4px 12px rgb(0 0 0 / 15%);
  :deep(.field) {
    justify-content: space-between;
  }
}
.area {
  position: relative;
  flex: 1;
  min-height: 0;
  // Tooltips and value labels may reach past the plot.
  overflow: visible;
}
.empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  margin: 0;
  padding: 0 1rem;
  color: var(--color-text-muted);
  text-align: center;
}
.footnote {
  margin: 0;
  color: var(--color-text-muted);
  font-size: 0.75rem;
}
.visually-hidden {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip-path: inset(50%);
  white-space: nowrap;
}
</style>
