<script lang="ts" setup>
import { computed, inject, ref, type Ref } from "vue";
import type { QueryType, TAB } from "../../composables/types";
import { useQueryStore } from "../../composables/queryStore";
import QueryStep from "./QueryStep.vue";
import QueryStepInput from "./QueryStepInput.vue";

const props = defineProps<{
  tab: TAB;
}>();

const queryStore = useQueryStore();

const queryId = inject<Ref<string>>("queryId");

const query = computed(() => {
  if (!queryId?.value) return null;
  return queryStore.getQuery(queryId.value)?.value;
});

const steps = computed(() => {
  if (!query.value) return [];
  return query.value.steps[props.tab];
});

// const content = ref("");
// const contentInput = ref<HTMLElement | null>(null);

// const updateContent = () => {
//   if (contentInput.value) {
//     content.value = contentInput.value.innerText;
//   }
// };

const addStep = (stepType: QueryType) => {
  console.log("Adding step:", stepType);
  if (!queryId?.value) return;
  queryStore.addQueryStep(queryId.value, props.tab, {
    id: `step-${Math.random().toString(36).substring(2, 9)}`,
    type: stepType,
  });
};
</script>

<template>
  <div class="query-builder">
    <div class="query-input" :class="[tab]">
      <div
        v-for="(step, index) in steps"
        :key="step.id"
        class="query-step-wrapper"
      >
        <!-- <QueryStepInput
          :prev-step="index > 0 ? steps[index - 1] : undefined"
        /> -->
        <QueryStep :step="step" />
      </div>
      <QueryStepInput
        :prev-step="steps.length > 0 ? steps[steps.length - 1] : undefined"
        @confirm-step="addStep"
      />
    </div>
    <button
      type="button"
      class="button"
      :class="[tab === 'exec_mut' ? 'button-warning' : 'button-primary']"
    >
      Run query
    </button>
  </div>
</template>

<style lang="less" scoped>
.query-builder {
  width: 100%;
  display: flex;
  button {
    border-top-left-radius: 0;
    border-bottom-left-radius: 0;
  }
}
.query-input {
  flex: 1;
  height: 100%;
  border: 1px solid var(--color-text);
  border-right: none;
  border-radius: 0.25rem 0 0 0.25rem;
  padding: 0.2rem;
  box-sizing: border-box;
  display: inline-block;
  transition: background-color 0.4s ease;
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
  &.context {
    background-color: var(--color-background-soft);
  }
  &.exec_mut {
    background-color: var(--orange-background);
  }
}
</style>
