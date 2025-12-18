<script lang="ts" setup>
import type { QueryStep } from "../../composables/types";

const props = defineProps<{
  step: QueryStep;
}>();
</script>

<template>
  <div class="query-step">
    <div class="label">{{ step.type }}</div>
  </div>
</template>

<style lang="less" scoped>
.query-step {
  display: inline-block;
  padding: 0rem 0.5rem;
  background-color: var(--color-background-soft);
  border: 1px solid var(--color-border);
  border-radius: 0.25rem;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  .label {
    font-weight: 500;
    color: var(--color-text);
  }
}
</style>
