<script lang="ts" setup>
import { computed, ref, useTemplateRef } from "vue";
import { useElementSize } from "@vueuse/core";
import type { ChartAxis } from "../../../utils/chartData";
import {
  CHAR_WIDTH,
  maxOf,
  minOf,
  formatCompact,
  formatValue,
  truncate,
  valueScale,
} from "../../../utils/chartScale";
import { stepMark } from "../../../utils/chartKeys";
import ChartTooltip from "./ChartTooltip.vue";

/** A line: its value at each X (a gap where it has none) and its colour. */
export type LineSeries = {
  key: string;
  label: string;
  /** The name at the line's end (else the label, shortened). */
  short?: string;
  color: string;
  values: (number | undefined)[];
};

const props = defineProps<{
  /** The groups along X, shared by every line. */
  xs: { key: string; label: string }[];
  series: LineSeries[];
  axis: ChartAxis;
  width: number;
  height: number;
  label: string;
}>();

const FONT = 11;
const PAD = 6;
/** Markers stay readable; with many points they show only on hover. */
const MARKER_RADIUS = 4;
const MAX_MARKERS = 30;
/** Lines named at their ends; with more, the legend alone names them. */
const MAX_END_LABELS = 4;
const END_LABEL_CHARS = 14;

const multi = computed(() => props.series.length > 1);
// The legend wraps as the names need; the plot takes the rest.
const legend = useTemplateRef<HTMLElement>("legend");
const { height: legendHeight } = useElementSize(legend, undefined, {
  box: "border-box",
});
const plotHeight = computed(() =>
  Math.max(1, props.height - (multi.value ? legendHeight.value : 0)),
);
const endName = (s: LineSeries) =>
  s.short ?? truncate(s.label, END_LABEL_CHARS);

const defined = (values: (number | undefined)[]) =>
  values.filter((v): v is number => v !== undefined);

const scale = computed(() =>
  valueScale(
    props.series.flatMap((s) => defined(s.values)),
    4,
    false,
  ),
);

/** Where each X sits: by value for numbers and dates, else evenly. */
const positions = computed(() => {
  const { xs, axis } = props;
  if (axis === "category" || xs.length < 2) {
    return xs.map((_, i) => (xs.length < 2 ? 0.5 : i / (xs.length - 1)));
  }
  const raw = xs.map((x) =>
    axis === "number" ? Number(x.label) : Date.parse(x.label),
  );
  const lo = minOf(raw);
  const span = maxOf(raw) - lo || 1;
  return raw.map((v) => (v - lo) / span);
});

const endLabelled = computed(
  () => multi.value && props.series.length <= MAX_END_LABELS,
);
/** Lines named at their end: those reaching the last X (others end inside). */
const named = computed(() =>
  endLabelled.value
    ? props.series.filter((s) => s.values.at(-1) !== undefined)
    : [],
);

const layout = computed(() => {
  const { width, xs, series } = props;
  const height = plotHeight.value;
  const { min, max, ticks } = scale.value;
  const left =
    Math.max(...ticks.map((t) => formatCompact(t).length)) * CHAR_WIDTH + PAD;
  const top = FONT + PAD;
  // Room for the X labels below the lowest tick label.
  const bottom = FONT * 2 + PAD;
  // Room for the line names, or the last value label and the last X label.
  const lastValue = defined(series[0]?.values ?? []).at(-1) ?? 0;
  const right = named.value.length
    ? Math.min(
        width / 3,
        maxOf(named.value.map((s) => endName(s).length)) * CHAR_WIDTH + 12,
      )
    : Math.max(24, (formatCompact(lastValue).length * CHAR_WIDTH) / 2 + 4);
  const plotW = Math.max(1, width - left - right);
  const plotH = Math.max(1, height - top - bottom);
  const xAt = (i: number) => left + (positions.value[i] ?? 0) * plotW;
  const y = (v: number) => top + ((max - v) / (max - min)) * plotH;

  const lines = series.map((s) => {
    const coords = s.values.map((v, i) =>
      v === undefined ? undefined : { x: xAt(i), y: y(v), value: v },
    );
    // A line joins its own points: an X only other lines have (no group of
    // this query there) is passed over. A single point shows as a marker.
    const points = coords.flatMap((c) => (c ? [c] : []));
    const path = points
      .map((c, i) => `${i ? "L" : "M"}${c.x},${c.y}`)
      .join(" ");
    return { ...s, coords, path, single: points.length === 1 };
  });

  // X labels by where they sit (numbers and dates are not evenly spread):
  // the first and the last, and between them each that keeps clear of both
  // neighbours.
  const room = (label: string) => Math.min(label.length, 12) * CHAR_WIDTH + 12;
  const at = xs.map((x, i) => ({ key: x.key, x: xAt(i), label: x.label }));
  const clear = (a: (typeof at)[number], b: (typeof at)[number]) =>
    Math.abs(b.x - a.x) >= Math.max(room(a.label), room(b.label));
  const first = at[0];
  const final = at.length > 1 ? at.at(-1) : undefined;
  const shownXs = first ? [first] : [];
  for (const x of at.slice(1, -1)) {
    const previous = shownXs.at(-1)!;
    if (clear(previous, x) && (!final || clear(x, final))) shownXs.push(x);
  }
  if (final && clear(shownXs.at(-1)!, final)) shownXs.push(final);

  // One line: values on its last and highest point. Several: their names
  // at their ends, nudged apart so they do not overlap.
  const valueLabels: { key: string; x: number; y: number; text: string }[] = [];
  const endLabels: {
    key: string;
    x: number;
    y: number;
    text: string;
    color: string;
  }[] = [];
  const only = lines[0];
  if (!multi.value && only) {
    const points = only.coords.flatMap((c) => (c ? [c] : []));
    const highest = points.reduce<(typeof points)[number] | undefined>(
      (a, b) => (!a || b.value > a.value ? b : a),
      undefined,
    );
    for (const c of new Set([points.at(-1), highest])) {
      if (c) {
        valueLabels.push({
          key: `${c.x}`,
          x: c.x,
          y: c.y - 8,
          text: formatCompact(c.value),
        });
      }
    }
  } else {
    const keys = new Set(named.value.map((s) => s.key));
    const ends = lines
      .flatMap((line) => {
        const end = line.coords.at(-1);
        return end && keys.has(line.key) ? [{ line, end }] : [];
      })
      .sort((a, b) => a.end.y - b.end.y);
    // Nudged down apart, then up again where that left the plot.
    const gap = FONT + 2;
    let previous = -Infinity;
    for (const { line, end } of ends) {
      const y = Math.max(end.y + 4, previous + gap);
      previous = y;
      endLabels.push({
        key: line.key,
        x: end.x + 8,
        y,
        text: endName(line),
        color: line.color,
      });
    }
    let below = top + plotH + 4 + gap;
    for (const label of [...endLabels].reverse()) {
      label.y = Math.min(label.y, below - gap);
      below = label.y;
    }
  }

  return {
    left,
    top,
    plotW,
    plotH,
    ticks: ticks.map((t) => ({ value: t, pos: y(t) })),
    lines,
    xAt,
    // The labels at either end stay within the plot.
    xLabels: shownXs.map((c, i) => ({
      key: c.key,
      x: c.x,
      text: truncate(c.label, 12),
      anchor:
        shownXs.length < 2
          ? "middle"
          : i === 0
            ? "start"
            : i === shownXs.length - 1
              ? "end"
              : "middle",
    })),
    valueLabels,
    endLabels,
    bottom: height - 2,
  };
});

const showMarkers = computed(() => props.xs.length <= MAX_MARKERS);

/** The X under the pointer or keyboard. */
const active = ref<number>();
/** The line whose legend entry is hovered, drawn over the dimmed rest. */
const highlighted = ref<string>();

const tip = computed(() => {
  const i = active.value;
  const x = i === undefined ? undefined : props.xs[i];
  if (i === undefined || !x) return undefined;
  const ys = layout.value.lines.flatMap((l) => {
    const c = l.coords[i];
    return c ? [c.y] : [];
  });
  const at = {
    x: layout.value.xAt(i),
    y: ys.length ? minOf(ys) : layout.value.top,
  };
  if (!multi.value) {
    const value = props.series[0]?.values[i];
    return {
      ...at,
      label: x.label,
      value: value === undefined ? "–" : formatValue(value),
    };
  }
  return {
    ...at,
    label: x.label,
    rows: props.series.map((s) => {
      const value = s.values[i];
      return {
        key: s.key,
        label: s.label,
        color: s.color,
        value: value === undefined ? "–" : formatValue(value),
      };
    }),
  };
});

/** The X nearest the pointer: a crosshair, not a tiny marker target. */
const onMove = (event: MouseEvent) => {
  const svg = event.currentTarget as SVGSVGElement;
  const px = event.clientX - svg.getBoundingClientRect().left;
  let nearest = 0;
  props.xs.forEach((_, i) => {
    const { xAt } = layout.value;
    if (Math.abs(xAt(i) - px) < Math.abs(xAt(nearest) - px)) nearest = i;
  });
  active.value = nearest;
};

const onKey = (event: KeyboardEvent) => {
  const next = stepMark(event.key, active.value, props.xs.length);
  if (next === undefined) return;
  event.preventDefault();
  active.value = next;
};

/** What the arrow keys read out: the X and each line's value there. */
const announcement = computed(() => {
  const t = tip.value;
  if (!t) return "";
  const values =
    "rows" in t
      ? t.rows.map((r) => `${r.label}: ${r.value}`).join(", ")
      : t.value;
  return `${t.label}: ${values}`;
});

const ariaLabel = computed(() => {
  const names = multi.value
    ? ` ${props.series.length} lines: ${props.series.map((s) => s.label).join(", ")}.`
    : "";
  return `${props.label}.${names} Use the arrow keys to read the points.`;
});
</script>

<template>
  <div class="chart-line" data-testid="chart-line">
    <ul
      v-if="multi"
      ref="legend"
      class="legend"
      data-testid="chart-line-legend"
    >
      <li
        v-for="s in series"
        :key="s.key"
        :class="{ dim: highlighted && highlighted !== s.key }"
        :title="s.label"
        @mouseenter="highlighted = s.key"
        @mouseleave="highlighted = undefined"
      >
        <span class="swatch" :style="{ background: s.color }" />
        <span class="name">{{ s.label }}</span>
      </li>
    </ul>
    <div class="plot">
      <svg
        :width="width"
        :height="plotHeight"
        :viewBox="`0 0 ${width} ${plotHeight}`"
        role="img"
        :aria-label="ariaLabel"
        tabindex="0"
        data-testid="chart-line-plot"
        @mousemove="onMove"
        @mouseleave="active = undefined"
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
          <text
            v-for="xl in layout.xLabels"
            :key="xl.key"
            class="x-label"
            :x="xl.x"
            :y="layout.bottom"
            :text-anchor="xl.anchor"
          >
            {{ xl.text }}
          </text>
        </g>
        <line
          v-if="tip"
          class="crosshair"
          :x1="tip.x"
          :x2="tip.x"
          :y1="layout.top"
          :y2="layout.top + layout.plotH"
        />
        <g
          v-for="(line, s) in layout.lines"
          :key="line.key"
          class="series"
          :class="{ dim: highlighted && highlighted !== line.key }"
          :data-testid="`chart-line-series-${s}`"
        >
          <path class="line" :d="line.path" :style="{ stroke: line.color }" />
          <template v-for="(c, index) in line.coords" :key="index">
            <circle
              v-if="c && (showMarkers || active === index || line.single)"
              class="marker"
              :class="{ active: active === index }"
              :cx="c.x"
              :cy="c.y"
              :r="active === index ? MARKER_RADIUS + 1 : MARKER_RADIUS"
              :style="{ fill: line.color }"
            />
          </template>
        </g>
        <text
          v-for="vl in layout.valueLabels"
          :key="vl.key"
          class="value"
          :x="vl.x"
          :y="vl.y"
          text-anchor="middle"
        >
          {{ vl.text }}
        </text>
        <text
          v-for="el in layout.endLabels"
          :key="el.key"
          class="end-label"
          :x="el.x"
          :y="el.y"
        >
          {{ el.text }}
        </text>
      </svg>
      <ChartTooltip v-if="tip" v-bind="tip" :width="width" />
      <p
        class="visually-hidden"
        aria-live="polite"
        data-testid="chart-line-announcement"
      >
        {{ announcement }}
      </p>
    </div>
  </div>
</template>

<style lang="less" scoped>
.chart-line {
  display: flex;
  flex-direction: column;
}
.plot {
  position: relative;
}
.visually-hidden {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip-path: inset(50%);
  white-space: nowrap;
}
.legend {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem 0.9rem;
  // Two rows at most; the names are in the tooltip and the table too.
  max-height: calc(2 * 1.2rem + 0.25rem);
  margin: 0;
  padding: 0 0 0.35rem;
  overflow: hidden;
  list-style: none;
  font-size: 0.75rem;
  li {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    min-width: 0;
    max-width: 14rem;
    color: var(--color-text);
    cursor: default;
    &.dim {
      opacity: 0.45;
    }
  }
  .swatch {
    flex: none;
    width: 0.9rem;
    height: 2px;
    border-radius: 1px;
  }
  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
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
.crosshair {
  stroke: var(--color-border-hover);
}
.series {
  transition: opacity 0.15s;
  &.dim {
    opacity: 0.2;
  }
}
.line {
  fill: none;
  stroke-width: 2;
  stroke-linejoin: round;
  stroke-linecap: round;
}
.marker {
  // A ring of the surface keeps markers apart from the line and each other.
  stroke: var(--graph-background);
  stroke-width: 2;
}
.value {
  fill: var(--color-text);
  font-variant-numeric: tabular-nums;
}
// Names wear text ink; the line beside them carries the colour.
.end-label {
  fill: var(--color-text);
}
</style>
