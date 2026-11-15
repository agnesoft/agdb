<script lang="ts" setup>
import { onMounted, ref } from "vue";
import type { QueryStepFieldValue } from "../../composables/types";
import { EDITABLE_VALUE_TYPES } from "../../utils/dbValueFields";

/**
 * A type select and a text input for one key or value, edited in place.
 * Enter commits, Esc cancels; the parent decides what leaving it means.
 */
const field = defineModel<QueryStepFieldValue>({ required: true });

const props = withDefaults(
  defineProps<{
    label: string;
    placeholder?: string;
    autofocus?: boolean;
    disabled?: boolean;
  }>(),
  { placeholder: undefined, autofocus: false, disabled: false },
);

const emit = defineEmits<{ commit: []; cancel: [] }>();

const input = ref<HTMLInputElement>();
onMounted(() => {
  if (props.autofocus) input.value?.focus();
});

defineExpose({ focus: () => input.value?.focus() });
</script>

<template>
  <span
    class="inline-field-editor"
    @keydown.enter.prevent="emit('commit')"
    @keydown.esc.stop.prevent="emit('cancel')"
  >
    <select
      v-model="field.selectedOption"
      class="type-select"
      :aria-label="`${label} type`"
      :disabled="disabled"
      data-testid="inline-type"
    >
      <option v-for="t in EDITABLE_VALUE_TYPES" :key="t" :value="t">
        {{ t }}
      </option>
    </select>
    <input
      ref="input"
      v-model="field.value"
      class="field-input"
      :aria-label="label"
      :placeholder="placeholder"
      :disabled="disabled"
      data-testid="inline-input"
    />
  </span>
</template>

<style lang="less" scoped>
.inline-field-editor {
  display: flex;
  gap: 0.2rem;
  min-width: 0;
}
.type-select {
  flex: 0 0 4.5rem;
  min-width: 0;
  font-size: 0.75rem;
}
.field-input {
  flex: 1;
  min-width: 0;
}
</style>
