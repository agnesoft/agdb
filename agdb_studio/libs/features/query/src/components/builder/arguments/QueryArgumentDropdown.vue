<script lang="ts" setup>
import { computed, ref } from "vue";
import { vOnClickOutside } from "@vueuse/components";

const buttonRef = ref<HTMLButtonElement | null>(null);

const props = withDefaults(
  defineProps<{
    options: readonly string[];
    modelValue?: string;
    shortcuts?: Record<string, string>;
  }>(),
  {
    modelValue: "",
    shortcuts: () => ({}),
  },
);

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
}>();

const isOpen = ref(false);

const hasShortcuts = computed(() =>
  props.options.some((option) => props.shortcuts[option] != null),
);

const closedLabel = computed(() => {
  if (!props.modelValue) return "";
  return props.shortcuts[props.modelValue] ?? props.modelValue;
});

const toggle = () => {
  isOpen.value = !isOpen.value;
};

const close = () => {
  isOpen.value = false;
};

const selectOption = (option: string) => {
  emit("update:modelValue", option);
  close();
};

defineExpose({
  focus: () => {
    buttonRef.value?.focus();
  },
});
</script>

<template>
  <div v-on-click-outside="close" class="arg-dropdown" @keydown.esc="close">
    <button
      ref="buttonRef"
      type="button"
      class="arg-select arg-select-trigger"
      :aria-expanded="isOpen"
      aria-haspopup="listbox"
      @click="toggle"
    >
      <span class="arg-select-label">{{ closedLabel }}</span>
      <span class="arg-select-arrow">▾</span>
    </button>

    <div v-if="isOpen" class="arg-options" role="listbox">
      <button
        v-for="opt in options"
        :key="opt"
        type="button"
        class="arg-option"
        :class="{ 'arg-option-selected': opt === modelValue }"
        role="option"
        :aria-selected="opt === modelValue"
        @click="selectOption(opt)"
      >
        <template v-if="hasShortcuts">
          <span class="arg-option-shortcut">{{ shortcuts[opt] ?? opt }}</span>
          <span class="arg-option-full">{{ opt }}</span>
        </template>
        <template v-else>
          <span class="arg-option-full">{{ opt }}</span>
        </template>
      </button>
    </div>
  </div>
</template>

<style lang="less" scoped>
.arg-dropdown {
  position: relative;
}

.arg-select {
  font-size: 0.75rem;
  padding: 0.15rem 0.3rem;
  border: 1px solid var(--color-border);
  border-radius: 0.2rem;
  background-color: var(--color-background);
  color: var(--color-text);
  cursor: pointer;
}

.arg-select-trigger {
  display: inline-flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  text-align: left;
}

.arg-select-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.arg-select-arrow {
  font-size: 0.65rem;
}

.arg-options {
  position: absolute;
  top: calc(100% + 0.15rem);
  left: 0;
  z-index: var(--z-index-dropdown);
  min-width: 100%;
  overflow: auto;
  padding: 0.15rem;
  border: 1px solid var(--color-border);
  border-radius: 0.2rem;
  background-color: var(--color-background);
  box-shadow: 0 0.25rem 0.75rem rgb(0 0 0 / 16%);
}

.arg-option {
  display: grid;
  grid-template-columns: 2rem 1fr;
  align-items: center;
  gap: 0.5rem;
  border: 0;
  border-radius: 0.2rem;
  background: transparent;
  color: var(--color-text);
  text-align: left;
  padding: 0.2rem 0.3rem;
  font-size: 0.75rem;
  font-family: inherit;
  cursor: pointer;
}

.arg-option:hover,
.arg-option-selected {
  background-color: var(--color-background-mute);
}

.arg-option-selected {
  color: var(--color-heading);
  border-color: var(--color-border-hover);
}

.arg-option-shortcut {
  opacity: 0.85;
}
</style>
