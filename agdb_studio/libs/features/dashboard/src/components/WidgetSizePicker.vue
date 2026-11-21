<script lang="ts" setup>
import { ref } from "vue";
import { WIDGET_COLS, WIDGET_ROWS, type WidgetSize } from "../types";

const props = defineProps<{ size: WidgetSize }>();

const emit = defineEmits<{
  resize: [size: WidgetSize];
}>();

/** The size under the pointer or focus, previewed in the grid. */
const preview = ref<WidgetSize>();

const shown = () => preview.value ?? props.size;
const covered = (size: WidgetSize) =>
  size.cols <= shown().cols && size.rows <= shown().rows;
const isCurrent = (size: WidgetSize) =>
  size.cols === props.size.cols && size.rows === props.size.rows;
</script>

<template>
  <div class="widget-size-picker">
    <div class="heading">
      <span>Size</span>
      <span class="label">{{ shown().cols }}×{{ shown().rows }}</span>
    </div>
    <div
      class="cells"
      role="group"
      aria-label="Widget size"
      :style="{ gridTemplateColumns: `repeat(${WIDGET_COLS.length}, 1fr)` }"
      @mouseleave="preview = undefined"
      @focusout="preview = undefined"
    >
      <template v-for="rows in WIDGET_ROWS" :key="rows">
        <button
          v-for="cols in WIDGET_COLS"
          :key="cols"
          type="button"
          class="cell"
          :class="{ covered: covered({ cols, rows }) }"
          :aria-label="`${cols} wide, ${rows} high`"
          :aria-pressed="isCurrent({ cols, rows })"
          :data-testid="`widget-size-${cols}x${rows}`"
          @mouseenter="preview = { cols, rows }"
          @focus="preview = { cols, rows }"
          @click="emit('resize', { cols, rows })"
        ></button>
      </template>
    </div>
  </div>
</template>

<style lang="less" scoped>
.widget-size-picker {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  padding: 0.35rem 0.5rem;
}
.heading {
  display: flex;
  justify-content: space-between;
  font-size: 0.85rem;
}
.label {
  color: var(--color-text-muted);
  font-variant-numeric: tabular-nums;
}
.cells {
  display: grid;
  gap: 0.2rem;
}
.cell {
  height: 1rem;
  padding: 0;
  border: 1px solid var(--color-border);
  border-radius: 0.2rem;
  background: var(--color-background-soft);
  cursor: pointer;
  &.covered {
    border-color: var(--color-background-active);
    background: var(--color-background-active);
    opacity: 0.75;
  }
  &:focus-visible {
    outline: 2px solid var(--color-border-hover);
    outline-offset: 1px;
  }
}
</style>
