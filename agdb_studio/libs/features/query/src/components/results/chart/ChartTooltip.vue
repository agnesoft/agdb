<script lang="ts" setup>
import { computed } from "vue";

const props = defineProps<{
  /** Where the mark is, in px within the chart. */
  x: number;
  y: number;
  /** The chart's width, to keep the tooltip inside it. */
  width: number;
  label: string;
  value?: string;
  /** A second line, e.g. a donut slice's share. */
  detail?: string;
  /** A value per series at the mark, e.g. each query's at a line chart's X. */
  rows?: { key: string; label: string; value: string; color: string }[];
}>();

/** About how tall the tooltip is, in px, to tell whether it fits above. */
const height = computed(
  () =>
    24 +
    (props.value === undefined ? 0 : 18) +
    (props.detail ? 16 : 0) +
    (props.rows?.length ?? 0) * 18,
);

/**
 * Anchored above the mark; near an edge it opens away from it, and below
 * the mark where there is no room above (the chart's top, in a small widget).
 */
const style = computed(() => {
  const edge = 80;
  let shift = "-50%";
  if (props.x < edge) shift = "-0.75rem";
  else if (props.x > props.width - edge) shift = "calc(-100% + 0.75rem)";
  const below = props.y < height.value;
  return {
    left: `${props.x}px`,
    top: `${props.y}px`,
    transform: `translate(${shift}, ${below ? "0.75rem" : "calc(-100% - 0.5rem)"})`,
  };
});
</script>

<template>
  <div
    class="chart-tooltip"
    role="tooltip"
    :style="style"
    data-testid="chart-tooltip"
  >
    <span class="label">{{ label }}</span>
    <span v-if="value !== undefined" class="value">{{ value }}</span>
    <span v-if="detail" class="detail">{{ detail }}</span>
    <ul v-if="rows" class="rows">
      <li v-for="row in rows" :key="row.key">
        <span class="swatch" :style="{ background: row.color }" />
        <span class="name">{{ row.label }}</span>
        <span class="value">{{ row.value }}</span>
      </li>
    </ul>
  </div>
</template>

<style lang="less" scoped>
.chart-tooltip {
  position: absolute;
  z-index: 2;
  display: flex;
  flex-direction: column;
  gap: 0.1rem;
  max-width: 18rem;
  padding: 0.35rem 0.55rem;
  border: 1px solid var(--color-border-hover);
  border-radius: 0.375rem;
  background: var(--color-background);
  box-shadow: 0 2px 8px rgb(0 0 0 / 15%);
  font-size: 0.8rem;
  pointer-events: none;
  white-space: nowrap;
}
.label {
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--color-text-muted);
}
.value {
  color: var(--color-heading);
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}
.detail {
  color: var(--color-text-muted);
}
.rows {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  gap: 0.1rem 0.4rem;
  margin: 0.15rem 0 0;
  padding: 0;
  list-style: none;
  li {
    display: contents;
  }
  .swatch {
    align-self: center;
    width: 0.6rem;
    height: 0.6rem;
    border-radius: 2px;
  }
  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--color-text);
  }
  .value {
    text-align: right;
  }
}
</style>
