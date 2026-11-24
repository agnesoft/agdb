<script lang="ts" setup>
import { computed, ref, useId } from "vue";
import type { ChartPoint } from "../../../utils/chartData";
import {
  OTHER_COLOR,
  formatCompact,
  formatValue,
  paletteColor,
} from "../../../utils/chartScale";
import { stepMark } from "../../../utils/chartKeys";
import ChartTooltip from "./ChartTooltip.vue";

const props = defineProps<{
  /** Positive values only; the caller explains the rest. */
  points: ChartPoint[];
  width: number;
  height: number;
  label: string;
}>();

/** Legend beside the ring in wide charts, below it in tall ones. */
const side = computed(() => props.width >= props.height * 1.1);
const LEGEND_SIDE = 180;
const LEGEND_ROW = 20;

const size = computed(() => {
  const w = side.value
    ? props.width - Math.min(LEGEND_SIDE, props.width / 2)
    : props.width;
  const h = side.value
    ? props.height
    : props.height - props.points.length * LEGEND_ROW - 8;
  return Math.max(40, Math.min(w, h));
});

const total = computed(() => props.points.reduce((sum, p) => sum + p.value, 0));

const percent = new Intl.NumberFormat(undefined, {
  style: "percent",
  maximumFractionDigits: 1,
});
const share = (value: number) => percent.format(value / total.value);

const color = (point: ChartPoint, index: number) =>
  point.other ? OTHER_COLOR : paletteColor(index);

const slices = computed(() => {
  const r = size.value / 2;
  const inner = r * 0.62;
  const c = r;
  let angle = -Math.PI / 2;
  return props.points.map((point, index) => {
    const sweep = (point.value / total.value) * Math.PI * 2;
    const start = angle;
    angle += sweep;
    const end = angle;
    const mid = (start + end) / 2;
    const at = (radius: number, a: number) =>
      `${c + radius * Math.cos(a)},${c + radius * Math.sin(a)}`;
    // A full ring cannot be one arc: two halves.
    const path =
      sweep >= Math.PI * 2 - 1e-9
        ? `M${at(r, 0)} A${r},${r} 0 1 1 ${at(r, Math.PI)} A${r},${r} 0 1 1 ${at(r, 0)} ` +
          `M${at(inner, 0)} A${inner},${inner} 0 1 0 ${at(inner, Math.PI)} A${inner},${inner} 0 1 0 ${at(inner, 0)} Z`
        : [
            `M${at(r, start)}`,
            `A${r},${r} 0 ${sweep > Math.PI ? 1 : 0} 1 ${at(r, end)}`,
            `L${at(inner, end)}`,
            `A${inner},${inner} 0 ${sweep > Math.PI ? 1 : 0} 0 ${at(inner, start)}`,
            "Z",
          ].join(" ");
    return {
      point,
      index,
      path,
      color: color(point, index),
      anchor: {
        x: c + ((r + inner) / 2) * Math.cos(mid),
        y: c + ((r + inner) / 2) * Math.sin(mid),
      },
    };
  });
});

const active = ref<number>();
// One tab stop for the chart; the arrow keys read its marks (see `stepMark`).
const id = useId();
const markId = (index: number) => `${id}-${index}`;
const onKey = (event: KeyboardEvent) => {
  const next = stepMark(event.key, active.value, slices.value.length);
  if (next === undefined) return;
  event.preventDefault();
  active.value = next;
};
const tip = computed(() => {
  const slice =
    active.value === undefined ? undefined : slices.value[active.value];
  return (
    slice && {
      ...slice.anchor,
      label: slice.point.label,
      value: formatValue(slice.point.value),
      detail: share(slice.point.value),
    }
  );
});
</script>

<template>
  <div class="chart-donut" :class="{ side }" data-testid="chart-donut">
    <div class="ring" :style="{ width: `${size}px`, height: `${size}px` }">
      <svg
        :width="size"
        :height="size"
        :viewBox="`0 0 ${size} ${size}`"
        role="group"
        :aria-label="`${label}. Use the arrow keys to read the slices.`"
        :aria-activedescendant="
          active === undefined ? undefined : markId(active)
        "
        tabindex="0"
        @keydown="onKey"
        @blur="active = undefined"
      >
        <path
          v-for="slice in slices"
          :id="markId(slice.index)"
          :key="slice.point.key"
          class="slice"
          :class="{
            active: active === slice.index,
            dim: active !== undefined && active !== slice.index,
          }"
          :d="slice.path"
          :style="{ fill: slice.color }"
          fill-rule="evenodd"
          role="img"
          :aria-label="`${slice.point.label}: ${formatValue(slice.point.value)}, ${share(slice.point.value)}`"
          :data-testid="`chart-slice-${slice.index}`"
          @mouseenter="active = slice.index"
          @mouseleave="active = undefined"
        />
      </svg>
      <div class="center" aria-hidden="true">
        <span class="total">{{ formatCompact(total) }}</span>
        <span class="caption">Total</span>
      </div>
      <ChartTooltip v-if="tip" v-bind="tip" :width="size" />
    </div>
    <ul class="legend" aria-label="Legend">
      <li
        v-for="slice in slices"
        :key="slice.point.key"
        :class="{ dim: active !== undefined && active !== slice.index }"
        :title="slice.point.label"
        @mouseenter="active = slice.index"
        @mouseleave="active = undefined"
      >
        <span class="swatch" :style="{ backgroundColor: slice.color }"></span>
        <span class="name">{{ slice.point.label }}</span>
        <span class="amount">{{ formatCompact(slice.point.value) }}</span>
      </li>
    </ul>
  </div>
</template>

<style lang="less" scoped>
.chart-donut {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  height: 100%;
  &.side {
    flex-direction: row;
    gap: 1.25rem;
  }
}
.ring {
  position: relative;
  flex: none;
}
svg {
  display: block;
  overflow: visible;
  outline: none;
  &:focus-visible {
    outline: 2px dashed var(--color-border-hover);
    outline-offset: 2px;
  }
}
.slice {
  // The surface gap between slices.
  stroke: var(--graph-background);
  stroke-width: 2;
  outline: none;
  transition: opacity 0.15s;
  &.dim {
    opacity: 0.45;
  }
  svg:focus-visible &.active {
    stroke: var(--color-text);
  }
}
.center {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  pointer-events: none;
}
.total {
  color: var(--color-heading);
  font-size: 1.1rem;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}
.caption {
  color: var(--color-text-muted);
  font-size: 0.7rem;
}
.legend {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
  min-width: 0;
  max-width: 180px;
  margin: 0;
  padding: 0;
  list-style: none;
  font-size: 0.8rem;
  li {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    height: 18px;
    transition: opacity 0.15s;
    &.dim {
      opacity: 0.45;
    }
  }
}
.swatch {
  flex: none;
  width: 0.65rem;
  height: 0.65rem;
  border-radius: 2px;
}
.name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--color-text);
}
.amount {
  margin-left: auto;
  padding-left: 0.5rem;
  color: var(--color-text-muted);
  font-variant-numeric: tabular-nums;
}
</style>
