<script lang="ts" setup>
import { formatValue } from "../../../utils/chartScale";

defineProps<{
  value: number;
  /** What the number is, e.g. "Sum of price". */
  label: string;
}>();
</script>

<template>
  <div class="chart-stat" data-testid="chart-stat">
    <span class="value" :title="formatValue(value)">{{
      formatValue(value)
    }}</span>
    <span class="label">{{ label }}</span>
  </div>
</template>

<style lang="less" scoped>
.chart-stat {
  // The number grows with the widget it is in.
  container-type: size;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.35rem;
  height: 100%;
  min-height: 4rem;
  overflow: hidden;
}
.value {
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--color-heading);
  font-size: clamp(1.5rem, 28cqmin, 4.5rem);
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  line-height: 1.1;
}
.label {
  color: var(--color-text-muted);
  font-size: 0.8rem;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}
</style>
