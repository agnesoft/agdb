<template>
  <div :class="['callout', `callout-${type}`]">
    <div class="callout-icon">
      <span v-if="type === 'info'">ℹ️</span>
      <span v-else-if="type === 'warning'">⚠️</span>
      <span v-else-if="type === 'error'">🚫</span>
      <span v-else-if="type === 'success'">✅</span>
      <span v-else>💡</span>
    </div>
    <div class="callout-content">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
type Props = {
  type?: 'info' | 'warning' | 'error' | 'success' | 'default'
}

withDefaults(defineProps<Props>(), {
  type: 'default'
})
</script>

<style scoped>
.callout {
  display: flex;
  gap: 0.75rem;
  padding: 1rem;
  margin: 1rem 0;
  border-radius: 0.5rem;
  border-left: 4px solid;
}

.callout-info {
  background-color: #eff6ff;
  border-color: #3b82f6;
  color: #1e40af;
}

.callout-warning {
  background-color: #fffbeb;
  border-color: #f59e0b;
  color: #92400e;
}

.callout-error {
  background-color: #fef2f2;
  border-color: #ef4444;
  color: #991b1b;
}

.callout-success {
  background-color: #f0fdf4;
  border-color: #10b981;
  color: #065f46;
}

.callout-default {
  background-color: #f9fafb;
  border-color: #6b7280;
  color: #374151;
}

.callout-icon {
  font-size: 1.25rem;
  flex-shrink: 0;
}

.callout-content {
  flex: 1;
}
</style></div>