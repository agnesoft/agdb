<script lang="ts" setup>
const props = defineProps<{
  hints: string[];
  activeIndex?: number;
}>();
defineEmits<{
  (e: "selectHint", hint: string): void;
}>();
</script>

<template>
  <div class="query-hinter">
    <div
      v-for="(hint, index) in hints"
      :key="index"
      class="hinter-item"
      :class="{ active: index === activeIndex }"
      @click="$emit('selectHint', hint)"
    >
      {{ hint }}
    </div>
  </div>
</template>

<style lang="less" scoped>
.query-hinter {
  position: absolute;
  top: 100%;
  left: 0;
  background: var(--color-background-soft);
  border: 1px solid var(--color-border);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  z-index: 10;
  max-height: 20rem;
  overflow-y: auto;
  width: max-content;
  min-width: 10rem;
}
.hinter-item {
  padding: 0.2rem 0.5rem;
  cursor: pointer;
  transition:
    background-color 0.2s ease,
    opacity 0.2s ease;
  &:hover {
    background-color: var(--color-background);
    opacity: 0.8;
  }
  &.active {
    background-color: var(--orange-2);
    color: var(--black);
  }
}
</style>
