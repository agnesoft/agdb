<script lang="ts" setup>
import { computed, inject, nextTick, ref, watch, type Ref } from "vue";
import { vOnClickOutside } from "@vueuse/components";
import { ClCloseMd } from "@kalimahapps/vue-icons";
import type { QueryStep, TAB } from "../../composables/types";
import { useQueryStore } from "../../composables/queryStore";
import { queryApiMock } from "../../mock/queryApiMock";
import QueryArgument from "./arguments/QueryArgument.vue";
import QueryArgumentDisplay from "./arguments/QueryArgumentDisplay.vue";

/* v8 ignore next -- @preserve */
const queryStore = useQueryStore();

const props = defineProps<{
  step: QueryStep;
}>();
const queryId = inject<Ref<string>>("queryId");
const tab = inject<Ref<TAB>>("activeTab");

const stepDef = computed(() => queryApiMock[props.step.type]);
const stepArguments = computed(() => stepDef.value?.arguments ?? null);
// Start in edit mode automatically when the step has no args yet (newly added).
const isEditingArgs = ref(!props.step.args?.length);

const removeStep = () => {
  if (!queryId?.value || !tab?.value) return;
  queryStore.deleteQueryStep(queryId.value, tab.value, props.step.id);
};
const closeEditing = () => {
  /* v8 ignore next -- @preserve */
  isEditingArgs.value = false;
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
    <div
      v-on-click-outside="closeEditing"
      class="label"
      :class="{ invalid: step.invalid }"
    >
      <span>{{ step.type }}</span>
      <template v-if="stepArguments">
        <QueryArgumentDisplay
          :arguments="stepArguments"
          :step="step"
          class="arg-display-trigger"
          @click="isEditingArgs = true"
        />
        <div v-if="isEditingArgs" class="arg-editor-popup">
          <QueryArgument
            :arguments="stepArguments"
            :step="step"
            :auto-focus="true"
          />
        </div>
      </template>
    </div>
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
  position: relative;
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
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

.arg-display-trigger {
  cursor: pointer;

  &:hover {
    color: var(--color-text);
  }
}

.arg-editor-popup {
  position: absolute;
  top: calc(100% + 0.25rem);
  left: 0;
  z-index: var(--z-index-dropdown);
  padding: 0.2rem;
  border: 1px solid var(--color-border);
  border-radius: 0.25rem;
  background-color: var(--color-background);
  box-shadow: 0 0.25rem 0.75rem rgb(0 0 0 / 16%);
}
</style>
