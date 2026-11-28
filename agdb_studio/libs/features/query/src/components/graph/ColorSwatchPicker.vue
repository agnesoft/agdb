<script lang="ts" setup>
import { ref } from "vue";
import { usePickerPopover } from "../../composables/pickerPopover";
import { LuPalette } from "@kalimahapps/vue-icons";
import type { PaletteSlot } from "../../utils/graphColorRules";

/**
 * Picks a palette colour (or none). Inline it shows the swatches directly;
 * otherwise a small dot opens them in a popover.
 */
const props = withDefaults(
  defineProps<{
    palette: readonly string[];
    modelValue?: PaletteSlot;
    label: string;
    inline?: boolean;
    /** Whether "no colour" can be picked. */
    clearable?: boolean;
  }>(),
  { modelValue: undefined, inline: false, clearable: true },
);

const emit = defineEmits<{
  "update:modelValue": [slot: PaletteSlot | undefined];
}>();

const root = ref<HTMLElement>();
const { open, onEscape } = usePickerPopover(root);

const pick = (slot: PaletteSlot | undefined) => {
  emit("update:modelValue", slot);
  open.value = false;
};
</script>

<template>
  <span ref="root" class="color-swatch-picker" @keydown.esc="onEscape">
    <button
      v-if="!inline"
      type="button"
      class="swatch-trigger"
      data-testid="swatch-trigger"
      :class="{ empty: props.modelValue === undefined }"
      :style="
        props.modelValue === undefined
          ? undefined
          : { backgroundColor: props.palette[props.modelValue] }
      "
      :title="label"
      :aria-label="label"
      :aria-expanded="open"
      @click.stop="open = !open"
    >
      <LuPalette v-if="props.modelValue === undefined" class="empty-icon" />
    </button>
    <span
      v-if="inline || open"
      class="swatches"
      :class="{ popover: !inline }"
      role="group"
      :aria-label="label"
    >
      <button
        v-for="(color, slot) in props.palette"
        :key="slot"
        type="button"
        class="swatch"
        data-testid="swatch"
        :class="{ current: slot === props.modelValue }"
        :style="{ backgroundColor: color }"
        :aria-label="`Colour ${slot + 1}`"
        :aria-pressed="slot === props.modelValue"
        @click.stop="pick(slot)"
      ></button>
      <button
        v-if="clearable"
        type="button"
        class="swatch swatch-none"
        data-testid="swatch-none"
        title="No colour"
        aria-label="No colour"
        :aria-pressed="props.modelValue === undefined"
        @click.stop="pick(undefined)"
      >
        ×
      </button>
    </span>
  </span>
</template>

<style lang="less" scoped>
.color-swatch-picker {
  position: relative;
  display: inline-flex;
  align-items: center;
}
.swatch-trigger {
  width: 0.7rem;
  height: 0.7rem;
  padding: 0;
  border-radius: 50%;
  border: 1px solid var(--color-border);
  cursor: pointer;
  &.empty {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.1rem;
    height: 1.1rem;
    border: none;
    border-radius: 0.2rem;
    background: none;
    color: var(--color-text-muted);
    opacity: 0.7;
  }
  .empty-icon {
    font-size: 0.9rem;
  }
  &:hover,
  &:focus-visible {
    opacity: 1;
  }
}
.swatches {
  display: inline-flex;
  gap: 0.25rem;
  &.popover {
    position: absolute;
    top: calc(100% + 0.25rem);
    left: 0;
    z-index: var(--z-index-dropdown);
    padding: 0.3rem;
    background-color: var(--color-background-mute);
    border: 1px solid var(--color-border);
    border-radius: 0.3rem;
    box-shadow: 0 4px 10px rgba(0, 0, 0, 0.25);
  }
}
.swatch {
  width: 1rem;
  height: 1rem;
  padding: 0;
  border-radius: 50%;
  // A ring in the surface colour keeps neighbouring swatches apart.
  border: 2px solid var(--color-background-mute);
  outline: 1px solid transparent;
  cursor: pointer;
  &.current,
  &[aria-pressed="true"] {
    outline-color: var(--color-text);
  }
}
.swatch-none {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.7rem;
  line-height: 1;
  color: var(--color-text-muted);
  background: none;
  border-color: var(--color-border);
}
</style>
