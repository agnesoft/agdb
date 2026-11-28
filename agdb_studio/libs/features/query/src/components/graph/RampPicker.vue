<script lang="ts" setup>
import { ref } from "vue";
import { usePickerPopover } from "../../composables/pickerPopover";
import {
  RAMP_NAMES,
  rampGradient,
  type RampName,
} from "../../utils/graphStyle";

/** Picks a gradient scale: a bar of the current one opens the others. */
const props = defineProps<{
  ramps: Readonly<Record<RampName, readonly string[]>>;
  modelValue: RampName;
  label: string;
}>();

const emit = defineEmits<{ "update:modelValue": [name: RampName] }>();

const NAMES: Record<RampName, string> = {
  teal: "Teal",
  purple: "Purple",
  rose: "Rose",
  multi: "Several colours",
};

const root = ref<HTMLElement>();
const { open, onEscape } = usePickerPopover(root);

const pick = (name: RampName) => {
  emit("update:modelValue", name);
  open.value = false;
};
</script>

<template>
  <span ref="root" class="ramp-picker" @keydown.esc="onEscape">
    <button
      type="button"
      class="ramp ramp-trigger"
      data-testid="ramp-trigger"
      :style="{ background: rampGradient(props.ramps[props.modelValue]) }"
      :title="`${label}: ${NAMES[props.modelValue]}`"
      :aria-label="`${label}: ${NAMES[props.modelValue]}`"
      :aria-expanded="open"
      @click.stop="open = !open"
    ></button>
    <span v-if="open" class="ramps" role="group" :aria-label="label">
      <button
        v-for="name in RAMP_NAMES"
        :key="name"
        type="button"
        class="ramp-option"
        data-testid="ramp-option"
        :aria-pressed="name === props.modelValue"
        @click.stop="pick(name)"
      >
        <span
          class="ramp"
          :style="{ background: rampGradient(props.ramps[name]) }"
        ></span>
        {{ NAMES[name] }}
      </button>
    </span>
  </span>
</template>

<style lang="less" scoped>
.ramp-picker {
  position: relative;
  display: inline-flex;
  align-items: center;
}
.ramp {
  width: 1.4rem;
  height: 0.8rem;
  padding: 0;
  border: none;
  border-radius: 0.2rem;
  cursor: pointer;
  &:focus-visible {
    outline: 1px solid var(--color-text);
    outline-offset: 1px;
  }
}
.ramp-option {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.15rem 0.3rem;
  border: 1px solid transparent;
  border-radius: 0.2rem;
  background: none;
  color: var(--color-text);
  white-space: nowrap;
  cursor: pointer;
  &:hover,
  &:focus-visible {
    background-color: var(--color-background-soft);
  }
  &[aria-pressed="true"] {
    border-color: var(--color-text-muted);
  }
}
.ramps {
  position: absolute;
  top: calc(100% + 0.25rem);
  left: 0;
  z-index: var(--z-index-dropdown);
  display: flex;
  flex-direction: column;
  gap: 0.1rem;
  padding: 0.3rem;
  background-color: var(--color-background-mute);
  border: 1px solid var(--color-border);
  border-radius: 0.3rem;
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.25);
  .ramp {
    width: 3rem;
  }
}
</style>
