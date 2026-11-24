<script lang="ts" setup>
import { computed, type Component } from "vue";
import {
  LuChartColumn,
  LuChartLine,
  LuChartPie,
  LuHash,
} from "@kalimahapps/vue-icons";
import {
  ADDITIVE_AGGREGATES,
  AGGREGATE_LABELS,
  CHART_AGGREGATES,
  type ChartAggregate,
  type ChartConfig,
  type ChartKind,
  type ResultKey,
} from "../../../utils/chartConfig";

const props = defineProps<{
  /** The chart as shown. */
  config: ChartConfig;
  /**
   * The chart as stored, when it differs from the one shown (a donut of an
   * average shows a sum): changes apply to it, so switching back to a kind
   * that can show it restores it.
   */
  saved?: ChartConfig;
  /** The property keys of the results. */
  keys: ResultKey[];
  /** How many queries the batch has, for lines split by query. */
  queries?: number;
}>();

const emit = defineEmits<{
  "update:config": [config: ChartConfig];
}>();

const KINDS: { kind: ChartKind; label: string; icon: Component }[] = [
  { kind: "bar", label: "Bar", icon: LuChartColumn },
  { kind: "line", label: "Line", icon: LuChartLine },
  { kind: "donut", label: "Donut", icon: LuChartPie },
  { kind: "stat", label: "Stat", icon: LuHash },
];

/** Sentinel option value of "Query of the batch" (property keys are free text). */
const BY_QUERY = "\u0000query";

/** Keys a saved config uses stay selectable while the results lack them. */
const withSaved = (keys: string[], saved: string | undefined) =>
  saved && !keys.includes(saved) ? [saved, ...keys] : keys;

const groupKeys = computed(() =>
  withSaved(
    props.keys.map((k) => k.key),
    props.config.groupBy.by === "property"
      ? props.config.groupBy.key
      : undefined,
  ),
);
const numericKeys = computed(() =>
  withSaved(
    props.keys.filter((k) => k.numeric).map((k) => k.key),
    props.config.measure.aggregate === "count"
      ? undefined
      : props.config.measure.key,
  ),
);
const isMissing = (key: string) => !props.keys.some((k) => k.key === key);

const groupValue = computed(() =>
  props.config.groupBy.by === "query" ? BY_QUERY : props.config.groupBy.key,
);
const measureKey = computed(() =>
  props.config.measure.aggregate === "count" ? "" : props.config.measure.key,
);

const update = (change: Partial<ChartConfig>) =>
  emit("update:config", { ...(props.saved ?? props.config), ...change });

/** Why an aggregate cannot be picked, if it cannot. */
const unavailable = (aggregate: ChartAggregate): string | undefined => {
  if (aggregate === "count") return undefined;
  if (!numericKeys.value.length) return "No numeric property in the results";
  if (
    props.config.kind === "donut" &&
    !ADDITIVE_AGGREGATES.includes(aggregate)
  ) {
    return "A donut shows parts of a total: use a bar chart for this";
  }
  return undefined;
};

const asAggregate = (value: string) =>
  CHART_AGGREGATES.find((a) => a === value);

const setGroup = (value: string) =>
  update({
    groupBy:
      value === BY_QUERY ? { by: "query" } : { by: "property", key: value },
  });

const setAggregate = (aggregate: ChartAggregate) => {
  if (aggregate === "count") return update({ measure: { aggregate } });
  // Keeps the measured key; a first pick takes the first numeric key.
  const key = measureKey.value || numericKeys.value[0];
  /* v8 ignore next -- @preserve: non-count options are disabled without keys */
  if (!key) return;
  update({ measure: { aggregate, key } });
};

const onAggregate = (value: string) => {
  const aggregate = asAggregate(value);
  /* v8 ignore next -- @preserve: the options are the aggregates */
  if (aggregate) setAggregate(aggregate);
};

const setMeasureKey = (key: string) => {
  // The stored aggregate: a donut of an average shows a sum, yet keeps it.
  const { measure } = props.saved ?? props.config;
  /* v8 ignore next -- @preserve: the key select is shown for non-count measures */
  if (measure.aggregate === "count") return;
  update({ measure: { aggregate: measure.aggregate, key } });
};

/** Why a chart type cannot be picked, if it cannot. */
const kindUnavailable = (kind: ChartKind): string | undefined =>
  kind === "line" && props.config.groupBy.by === "query"
    ? "A line runs along ordered values: group by a property for it"
    : undefined;

/** A line per query, offered for batches (and kept while one is set). */
const showSplit = computed(
  () =>
    props.config.kind === "line" &&
    props.config.groupBy.by === "property" &&
    ((props.queries ?? 0) > 1 || !!props.config.split),
);

const SPLIT_BY_QUERY = "query";

const setSplit = (value: string) => {
  const { split: _, ...rest } = props.saved ?? props.config;
  emit(
    "update:config",
    value === SPLIT_BY_QUERY ? { ...rest, split: { by: "query" } } : rest,
  );
};

const selectValue = (event: Event) => (event.target as HTMLSelectElement).value;
</script>

<template>
  <div class="chart-settings" data-testid="chart-settings">
    <div class="kinds" role="group" aria-label="Chart type">
      <button
        v-for="{ kind, label, icon } in KINDS"
        :key="kind"
        type="button"
        class="kind"
        :class="{ active: config.kind === kind }"
        :aria-pressed="config.kind === kind"
        :disabled="!!kindUnavailable(kind)"
        :title="kindUnavailable(kind) ?? label"
        :data-testid="`chart-kind-${kind}`"
        @click="update({ kind })"
      >
        <component :is="icon" aria-hidden="true" />
        <span>{{ label }}</span>
      </button>
    </div>
    <label class="field">
      <span>Show</span>
      <select
        :value="config.measure.aggregate"
        data-testid="chart-aggregate"
        @change="onAggregate(selectValue($event))"
      >
        <option
          v-for="aggregate in CHART_AGGREGATES"
          :key="aggregate"
          :value="aggregate"
          :disabled="!!unavailable(aggregate)"
          :title="unavailable(aggregate)"
        >
          {{ AGGREGATE_LABELS[aggregate] }}
        </option>
      </select>
    </label>
    <label v-if="config.measure.aggregate !== 'count'" class="field">
      <span>of</span>
      <select
        :value="measureKey"
        data-testid="chart-measure-key"
        @change="setMeasureKey(selectValue($event))"
      >
        <option v-for="key in numericKeys" :key="key" :value="key">
          {{ key }}{{ isMissing(key) ? " (not in results)" : "" }}
        </option>
      </select>
    </label>
    <label v-if="config.kind !== 'stat'" class="field">
      <span>by</span>
      <select
        :value="groupValue"
        data-testid="chart-group-by"
        @change="setGroup(selectValue($event))"
      >
        <option :value="BY_QUERY">Query of the batch</option>
        <option v-for="key in groupKeys" :key="key" :value="key">
          {{ key }}{{ isMissing(key) ? " (not in results)" : "" }}
        </option>
      </select>
    </label>
    <label v-if="showSplit" class="field">
      <span>Lines</span>
      <select
        :value="config.split ? SPLIT_BY_QUERY : ''"
        data-testid="chart-split"
        @change="setSplit(selectValue($event))"
      >
        <option value="">One</option>
        <option :value="SPLIT_BY_QUERY">One per query</option>
      </select>
    </label>
    <p
      v-if="
        config.measure.aggregate === 'count' &&
        !numericKeys.length &&
        keys.length
      "
      class="note"
    >
      No numeric properties to sum or average.
    </p>
  </div>
</template>

<style lang="less" scoped>
.chart-settings {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem 0.75rem;
  font-size: 0.85rem;
}
.kinds {
  display: flex;
  padding: 2px;
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  background: var(--color-background-soft);
}
.kind {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  height: 1.75rem;
  padding: 0 0.55rem;
  border: none;
  border-radius: 0.25rem;
  background: transparent;
  color: var(--color-text-muted);
  font-size: 0.8rem;
  cursor: pointer;
  &:hover:not(:disabled) {
    color: var(--color-text);
  }
  &:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  &.active {
    background: var(--color-background-mute);
    color: var(--color-text);
    box-shadow: 0 0 0 1px var(--color-border-hover);
  }
  &:focus-visible {
    outline: 2px solid var(--color-border-hover);
    outline-offset: 1px;
  }
}
.field {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  color: var(--color-text-muted);
  select {
    height: 1.75rem;
    max-width: 12rem;
    padding: 0 0.35rem;
    border: 1px solid var(--color-border);
    border-radius: 0.25rem;
    background: var(--color-background-soft);
    color: var(--color-text);
    font-size: 0.85rem;
    &:hover,
    &:focus {
      outline: none;
      border-color: var(--color-border-hover);
    }
  }
}
.note {
  margin: 0;
  color: var(--color-text-muted);
  font-size: 0.8rem;
}
</style>
