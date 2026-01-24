<script lang="ts" setup>
import { ClCloseMd } from "@kalimahapps/vue-icons";
import type { QueryStep, TAB } from "../../composables/types";
import { useQueryStore } from "../../composables/queryStore";
import { inject, type Ref } from "vue";

const queryStore = useQueryStore();

const props = defineProps<{
  step: QueryStep;
}>();
const queryId = inject<Ref<string>>("queryId");
const tab = inject<Ref<TAB>>("activeTab");

const removeStep = () => {
  if (!queryId?.value || !tab?.value) return;
  queryStore.deleteQueryStep(queryId.value, tab.value, props.step.id);
};
</script>

<template>
  <div class="query-step">
    <div class="controls">
      <button
        type="button"
        class="button button-danger remove-step-button"
        title="Remove step"
        @click="removeStep"
      >
        <ClCloseMd class="remove-query-icon" />
      </button>
    </div>
    <div class="label" :class="{ invalid: step.invalid }">{{ step.type }}</div>
  </div>
</template>

<style lang="less" scoped>
.query-step {
  display: inline-block;
  position: relative;
  &:hover .controls {
    z-index: 10;
    visibility: visible;
    opacity: 1;
  }
}
.label {
  font-weight: 500;
  color: var(--color-text);
  padding: 0rem 0.5rem;
  background-color: var(--color-background-soft);
  border: 1px solid var(--color-border);
  border-radius: 0.25rem;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  transition: border-color 0.2s ease;
  &.invalid {
    border-color: var(--error-color);
  }
}

.controls {
  position: absolute;
  top: -1rem;
  right: -0.6rem;
  visibility: hidden;
  z-index: -1;
  transition: opacity 0.2s ease;
  opacity: 0;
}
.remove-step-button {
  padding: 0.2rem;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
