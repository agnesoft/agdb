<script lang="ts" setup>
const props = defineProps<{
  label: string;
  value: number;
}>();
</script>

<template>
  <div class="result-scalar" data-testid="result-scalar">
    <span class="label">{{ props.label }}</span>
    <span class="value">{{ props.value.toLocaleString() }}</span>
  </div>
</template>

<style lang="less" scoped>
.result-scalar {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  min-height: 20rem;
  height: 100%;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  background-color: var(--graph-background);
}
.label {
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.value {
  font-size: 3rem;
  font-weight: bold;
  color: var(--color-heading);
}
</style>
