<script lang="ts" setup>
import { computed, ref, useId } from "vue";
import type { ChartPoint } from "../../../utils/chartData";
import {
  CHAR_WIDTH,
  OTHER_COLOR,
  barPath,
  formatCompact,
  formatValue,
  paletteColor,
  truncate,
  valueScale,
} from "../../../utils/chartScale";
import { stepMark } from "../../../utils/chartKeys";
import ChartTooltip from "./ChartTooltip.vue";

const props = defineProps<{
  points: ChartPoint[];
  width: number;
  height: number;
  /** Read out with the chart, e.g. "Count by type". */
  label: string;
}>();

/** Space between bars, and between a bar and its label. */
const GAP = 2;
const LABEL_PAD = 6;
/** Bars never grow thicker than this, so few bars don't look like blocks. */
const MAX_THICKNESS = 40;
const FONT = 11;

const maxLabel = computed(() =>
  Math.max(...props.points.map((p) => p.label.length)),
);

/**
 * Columns in wide charts with few short labels; otherwise horizontal bars,
 * whose labels have room in a column of their own.
 */
const vertical = computed(
  () =>
    props.width >= props.height * 1.2 &&
    maxLabel.value * CHAR_WIDTH <= props.width / props.points.length - 4,
);

const scale = computed(() => valueScale(props.points.map((p) => p.value)));

const layout = computed(() => {
  const { width, height, points } = props;
  const { min, max } = scale.value;
  const valueLabels = points.map((p) => formatCompact(p.value));
  const valueLabelWidth =
    Math.max(...valueLabels.map((l) => l.length)) * CHAR_WIDTH + LABEL_PAD;
    const negative = points.some((p) => p.value < 0);
    const positive = points.some((p) => p.value > 0);
  if (vertical.value) {
    const axisWidth =
      Math.max(...scale.value.ticks.map((t) => formatCompact(t).length)) *
        CHAR_WIDTH +
      LABEL_PAD;
    const left = axisWidth;
    const top = FONT + LABEL_PAD;
    const bottom = FONT + LABEL_PAD + (negative ? FONT + 4 : 0);
    const plotW = Math.max(1, width - left - 4);
    const plotH = Math.max(1, height - top - bottom);
    const y = (v: number) => top + ((max - v) / (max - min)) * plotH;
    const band = plotW / points.length;
    const thickness = Math.max(
      1,
      Math.min(MAX_THICKNESS, band * 0.7, band - GAP),
    );
    const showValues = thickness >= valueLabelWidth - LABEL_PAD;
    return {
      left,
      top,
      plotW,
      plotH,
      ticks: scale.value.ticks.map((t) => ({ value: t, pos: y(t) })),
      zero: y(0),
      bars: points.map((point, i) => {
        const cross = left + i * band + (band - thickness) / 2;
        const end = y(point.value);
        return {
          point,
          path: barPath(true, cross, thickness, y(0), end),
          hit: { x: left + i * band, y: top, width: band, height: plotH },
          // Above the value label, which the tooltip would cover.
          center: {
            x: cross + thickness / 2,
            y: Math.min(end, y(0)) - (showValues ? FONT + 4 : 0),
          },
          label: {
            text: truncate(point.label, Math.floor(band / CHAR_WIDTH)),
            x: cross + thickness / 2,
            y: height - 2,
            anchor: "middle",
          },
          value: showValues
            ? {
                text: valueLabels[i],
                x: cross + thickness / 2,
                y: point.value < 0 ? end + FONT + 2 : end - 4,
                anchor: "middle",
              }
            : undefined,
        };
      }),
    };
  }
  // Labels get up to 40 % of the width, values room after the longest bar.
  const left = Math.min(width * 0.4, maxLabel.value * CHAR_WIDTH + LABEL_PAD);
  const start = left + (negative ? FONT + 4 : 0);
  const right = positive ? valueLabelWidth : 4;
  const plotW = Math.max(1, width - start - right);
  const plotH = Math.max(1, height);
  const x = (v: number) => start + ((v - min) / (max - min)) * plotW;
  const band = plotH / points.length;
  const thickness = Math.max(
    1,
    Math.min(MAX_THICKNESS / 1.5, band * 0.7, band - GAP),
  );
  return {
    left,
    top: 0,
    plotW,
    plotH,
    ticks: [],
    zero: x(0),
    bars: points.map((point, i) => {
      const cross = i * band + (band - thickness) / 2;
      const end = x(point.value);
      const middle = cross + thickness / 2 + FONT / 2 - 2;
      return {
        point,
        path: barPath(false, cross, thickness, x(0), end),
        hit: { x: 0, y: i * band, width, height: band },
        center: { x: Math.max(end, x(0)), y: cross },
        label: {
          text: truncate(
            point.label,
            Math.floor((left - LABEL_PAD) / CHAR_WIDTH),
          ),
          x: left - LABEL_PAD,
          y: middle,
          anchor: "end",
        },
        value:
          band >= FONT
            ? {
                text: valueLabels[i],
                x: point.value < 0 ? end - 4 : end + 4,
                y: middle,
                anchor: point.value < 0 ? "end" : "start",
              }
            : undefined,
      };
    }),
  };
});

const active = ref<number>();
// One tab stop for the chart; the arrow keys read its marks (see `stepMark`).
const id = useId();
const markId = (index: number) => `${id}-${index}`;
const onKey = (event: KeyboardEvent) => {
  const next = stepMark(event.key, active.value, layout.value.bars.length);
  if (next === undefined) return;
  event.preventDefault();
  active.value = next;
};
const tip = computed(() => {
  const bar =
    active.value === undefined ? undefined : layout.value.bars[active.value];
  return (
    bar && {
      ...bar.center,
      label: bar.point.label,
      value: formatValue(bar.point.value),
    }
  );
});

const fill = (point: ChartPoint) =>
  point.other ? OTHER_COLOR : paletteColor(0);
</script>

<template>
  <div class="chart-bar" data-testid="chart-bar">
    <svg
      :width="width"
      :height="height"
      :viewBox="`0 0 ${width} ${height}`"
      role="group"
      :aria-label="`${label}. Use the arrow keys to read the bars.`"
      :aria-activedescendant="active === undefined ? undefined : markId(active)"
      tabindex="0"
      @keydown="onKey"
      @blur="active = undefined"
    >
      <g class="grid" aria-hidden="true">
        <template v-for="tick in layout.ticks" :key="tick.value">
          <line
            :x1="layout.left"
            :x2="layout.left + layout.plotW"
            :y1="tick.pos"
            :y2="tick.pos"
          />
          <text :x="layout.left - 6" :y="tick.pos + 4" text-anchor="end">
            {{ formatCompact(tick.value) }}
          </text>
        </template>
      </g>
      <g
        v-for="(bar, index) in layout.bars"
        :id="markId(index)"
        :key="bar.point.key"
        class="bar"
        :class="{
          active: active === index,
          dim: active !== undefined && active !== index,
        }"
        role="img"
        :aria-label="`${bar.point.label}: ${formatValue(bar.point.value)}`"
        :data-testid="`chart-bar-${index}`"
        @mouseenter="active = index"
        @mouseleave="active = undefined"
      >
        <rect class="hit" v-bind="bar.hit" />
        <path :d="bar.path" :style="{ fill: fill(bar.point) }" />
        <text
          class="category"
          :x="bar.label.x"
          :y="bar.label.y"
          :text-anchor="bar.label.anchor"
        >
          {{ bar.label.text }}
        </text>
        <text
          v-if="bar.value"
          class="value"
          :x="bar.value.x"
          :y="bar.value.y"
          :text-anchor="bar.value.anchor"
        >
          {{ bar.value.text }}
        </text>
      </g>
      <line
        v-if="vertical"
        class="baseline"
        :x1="layout.left"
        :x2="layout.left + layout.plotW"
        :y1="layout.zero"
        :y2="layout.zero"
      />
      <line
        v-else
        class="baseline"
        :x1="layout.zero"
        :x2="layout.zero"
        :y1="0"
        :y2="layout.plotH"
      />
    </svg>
    <ChartTooltip v-if="tip" v-bind="tip" :width="width" />
  </div>
</template>

<style lang="less" scoped>
.chart-bar {
  position: relative;
}
svg {
  display: block;
  overflow: visible;
  font-size: 11px;
  outline: none;
  &:focus-visible {
    outline: 2px dashed var(--color-border-hover);
    outline-offset: 2px;
  }
}
.grid {
  line {
    stroke: var(--color-border);
    stroke-dasharray: 2 3;
  }
  text {
    fill: var(--color-text-muted);
  }
}
.baseline {
  stroke: var(--color-border-hover);
}
.bar {
  outline: none;
  cursor: default;
  path {
    transition: opacity 0.15s;
  }
  &.dim path {
    opacity: 0.45;
  }
  svg:focus-visible &.active .hit {
    stroke: var(--color-border-hover);
    stroke-dasharray: 3 2;
  }
}
.hit {
  fill: transparent;
}
.category {
  fill: var(--color-text-muted);
}
.value {
  fill: var(--color-text);
  font-variant-numeric: tabular-nums;
}
</style>
