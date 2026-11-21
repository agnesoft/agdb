<script lang="ts" setup>
import { ref, useTemplateRef } from "vue";
import { onClickOutside } from "@vueuse/core";
import { WIDGET_COLS, WIDGET_ROWS, type WidgetSize } from "../types";

const props = defineProps<{ size: WidgetSize }>();

const emit = defineEmits<{
  resize: [size: WidgetSize];
}>();

const open = ref(false);
/** The size under the pointer or focus, previewed in the grid. */
const preview = ref<WidgetSize>();

const close = () => {
  open.value = false;
  preview.value = undefined;
};

const root = useTemplateRef<HTMLElement>("root");
onClickOutside(root, close);

const shown = () => preview.value ?? props.size;
const covered = (size: WidgetSize) =>
  size.cols <= shown().cols && size.rows <= shown().rows;
const isCurrent = (size: WidgetSize) =>
  size.cols === props.size.cols && size.rows === props.size.rows;

const pick = (size: WidgetSize) => {
  emit("resize", size);
  close();
};
</script>

<template>
  <span ref="root" class="widget-size-picker" @keydown.esc.stop="close">
    <button
      type="button"
      class="toggle"
      :title="`Size: ${size.cols} wide, ${size.rows} high`"
      :aria-label="`Size: ${size.cols} wide, ${size.rows} high`"
      :aria-expanded="open"
      data-testid="widget-size"
      @click="open ? close() : (open = true)"
    >
      {{ size.cols }}×{{ size.rows }}
    </button>
    <div
      v-if="open"
      class="cells"
      role="group"
      aria-label="Widget size"
      :style="{ gridTemplateColumns: `repeat(${WIDGET_COLS.length}, 1fr)` }"
      @mouseleave="preview = undefined"
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
          @click="pick({ cols, rows })"
        ></button>
      </template>
      <span class="label">{{ shown().cols }}×{{ shown().rows }}</span>
    </div>
  </span>
</template>

<style lang="less" scoped>
.widget-size-picker {
  position: relative;
  display: inline-flex;
}
.toggle {
  min-width: 2.25rem;
  height: 1.75rem;
  padding: 0 0.35rem;
  border: none;
  border-radius: 0.25rem;
  background: transparent;
  color: var(--color-text);
  font-size: 0.75rem;
  font-variant-numeric: tabular-nums;
  cursor: pointer;
  &:hover,
  &[aria-expanded="true"] {
    background: var(--color-background-mute);
  }
}
.cells {
  position: absolute;
  top: calc(100% + 0.25rem);
  right: 0;
  z-index: var(--z-index-dropdown);
  display: grid;
  gap: 0.2rem;
  padding: 0.4rem;
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  background: var(--color-background);
}
.cell {
  width: 1.25rem;
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
.label {
  grid-column: 1 / -1;
  text-align: center;
  font-size: 0.75rem;
  color: var(--color-text-muted);
  font-variant-numeric: tabular-nums;
}
</style>
